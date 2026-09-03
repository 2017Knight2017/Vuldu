use crate::{
	ActionContext, ActionFunc, Active, CurrentSector, Database, MobjAi, MobjFlags, MobjNum,
	MobjType, MonsterRotation, PLAYERHEIGHT, Pass, PlayerMarker, Position, Random, SfxEvent,
	SpriteAnimation, Target, Traversal, in_fov, set_mobj_state,
};
use hecs::{CommandBuffer, Entity, World};
use wad_parser::{Level, LineFlags, LineId, NF_SUBSECTOR, SubsectorId, to_u64};

#[derive(Debug, Clone, Copy)]
struct DivLine {
	pub x: f32,
	pub z: f32,
	pub dx: f32,
	pub dz: f32,
}

fn p_divline_side(x: f32, y: f32, node: &DivLine) -> i32 {
	if node.dx == 0.0 {
		if x == node.x {
			return 2;
		}
		if x <= node.x {
			return if node.dz > 0.0 { 1 } else { 0 };
		}
		return if node.dz < 0.0 { 1 } else { 0 };
	}

	if node.dz == 0.0 {
		if y == node.z {
			return 2;
		}
		if y <= node.z {
			return if node.dx < 0.0 { 1 } else { 0 };
		}
		return if node.dx > 0.0 { 1 } else { 0 };
	}

	let dx = x - node.x;
	let dy = y - node.z;

	let left = node.dz * dx;
	let right = dy * node.dx;

	if right < left {
		0 // front side
	} else if (left - right).abs() < f32::EPSILON {
		2 // on line
	} else {
		1 // back side
	}
}

fn p_intercept_vector2(v2: &DivLine, v1: &DivLine) -> f32 {
	let den = v1.dz * v2.dx - v1.dx * v2.dz;

	if den == 0.0 {
		return 0.0;
	}

	let num = (v1.x - v2.x) * v1.dz + (v2.z - v1.z) * v1.dx;
	num / den
}

struct SightContextInner<'a> {
	strace: DivLine,
	t2x: f32,
	t2z: f32,
	sight_ystart: f32,
	top_slope: f32,
	bottom_slope: f32,
	pass: Pass<'a>,
}

impl<'a> SightContextInner<'a> {
	pub fn new(
		t1_pos: (f32, f32, f32),
		t1_height: f32,
		t2_pos: (f32, f32, f32),
		t2_height: f32,
		pass: Pass<'a>,
	) -> Self {
		let sight_ystart = t1_pos.1 + t1_height - (t1_height * 0.25);
		let top_slope = (t2_pos.1 + t2_height) - sight_ystart;
		let bottom_slope = t2_pos.1 - sight_ystart;

		let strace = DivLine {
			x: t1_pos.0,
			z: t1_pos.2,
			dx: t2_pos.0 - t1_pos.0,
			dz: t2_pos.2 - t1_pos.2,
		};

		Self {
			strace,
			t2x: t2_pos.0,
			t2z: t2_pos.2,
			sight_ystart,
			top_slope,
			bottom_slope,
			pass,
		}
	}

	pub fn cross_subsector(&mut self, subsector_id: SubsectorId, level: &Level) -> bool {
		let subsector = &level.geom.subsectors[subsector_id.0];

		let min = subsector.firstseg as usize;
		let max = (subsector.firstseg + subsector.numsegs) as usize;

		for i in min..max {
			let seg = &level.geom.segs[i];

			let line_id = LineId(seg.linedef as usize);
			let line = &level.geom.lines[line_id.0];

			if !self.pass.visit_line(line_id) {
				continue;
			}

			let seg_v1_x = level.geom.vertices[seg.v1 as usize].0;
			let seg_v1_z = level.geom.vertices[seg.v1 as usize].1;
			let seg_v2_x = level.geom.vertices[seg.v2 as usize].0;
			let seg_v2_z = level.geom.vertices[seg.v2 as usize].1;

			let s1 = p_divline_side(seg_v1_x, seg_v1_z, &self.strace);
			let s2 = p_divline_side(seg_v2_x, seg_v2_z, &self.strace);

			if s1 == s2 {
				continue;
			}

			let divl = DivLine {
				x: seg_v1_x,
				z: seg_v1_z,
				dx: seg_v2_x - seg_v1_x,
				dz: seg_v2_z - seg_v1_z,
			};

			let s1 = p_divline_side(self.strace.x, self.strace.z, &divl);
			let s2 = p_divline_side(self.t2x, self.t2z, &divl);

			if s1 == s2 {
				continue;
			}

			if !line.flags.contains(LineFlags::TWO_SIDED) {
				return false;
			}

			let (Some(front_side_id), Some(back_side_id)) = line.sides else {
				return false;
			};

			// We already checked whether sides are None,
			// so unwrap() is safe here.
			let open = level.get_opening(line_id).unwrap();

			if open.floor_high >= open.top {
				return false;
			}

			let front_sector = &level.state.sectors[level.geom.sides[front_side_id.0].sector.0];
			let back_sector = &level.state.sectors[level.geom.sides[back_side_id.0].sector.0];

			let frac = p_intercept_vector2(&self.strace, &divl);

			if front_sector.floor_h != back_sector.floor_h {
				let slope = (open.floor_high - self.sight_ystart) / frac;
				if slope > self.bottom_slope {
					self.bottom_slope = slope;
				}
			}

			if front_sector.ceil_h != back_sector.ceil_h {
				let slope = (open.top - self.sight_ystart) / frac;
				if slope < self.top_slope {
					self.top_slope = slope;
				}
			}

			if self.top_slope <= self.bottom_slope {
				return false;
			}
		}

		true
	}

	pub fn cross_bsp_node(&mut self, bspnum: usize, level: &Level) -> bool {
		if bspnum & NF_SUBSECTOR != 0 {
			if bspnum == u16::MAX as usize {
				return self.cross_subsector(SubsectorId(0), level);
			} else {
				return self.cross_subsector(SubsectorId(bspnum & !NF_SUBSECTOR), level);
			}
		}

		let bsp = &level.geom.nodes[bspnum];
		let bsp_divline = DivLine {
			x: bsp.x as f32,
			z: bsp.y as f32,
			dx: bsp.dx as f32,
			dz: bsp.dy as f32,
		};

		let mut side = p_divline_side(self.strace.x, self.strace.z, &bsp_divline);
		if side == 2 {
			side = 0;
		}

		if !self.cross_bsp_node(bsp.children[side as usize] as usize, level) {
			return false;
		}

		if side == p_divline_side(self.t2x, self.t2z, &bsp_divline) {
			return true;
		}

		self.cross_bsp_node(bsp.children[(side ^ 1) as usize] as usize, level)
	}
}

pub(crate) struct SightContext<'a> {
	pub(crate) pos: Position,
	pub(crate) cur_sector: CurrentSector,
	pub(crate) height: f32,
	pub(crate) target_pos: Position,
	pub(crate) target_sector: CurrentSector,
	pub(crate) target_height: f32,
	pub(crate) level: &'a Level,
}

pub(crate) fn p_check_sight(ctx: &SightContext, traversal: &mut Traversal) -> bool {
	if ctx.level.geom.reject_table.is_rejected(
		ctx.cur_sector.0,
		ctx.target_sector.0,
		ctx.level.state.sectors.len(),
	) {
		return false;
	}

	let pass = traversal.begin();

	let mut inner_ctx = SightContextInner::new(
		(ctx.pos.x, ctx.pos.y, ctx.pos.z),
		ctx.height,
		(ctx.target_pos.x, ctx.target_pos.y, ctx.target_pos.z),
		ctx.target_height,
		pass,
	);

	if ctx.level.geom.nodes.is_empty() {
		return true;
	}

	let head_node = ctx.level.geom.nodes.len() - 1;
	inner_ctx.cross_bsp_node(head_node, ctx.level)
}

pub(crate) struct LookContext<'a> {
	pub(crate) world: &'a World,
	pub(crate) ent: Entity,
	pub(crate) db: &'a Database,
	pub(crate) level: &'a mut Level,
	pub(crate) cmd: &'a mut CommandBuffer,
	pub(crate) random: &'a mut Random,
	pub(crate) audio: &'a mut Vec<SfxEvent>,
	pub(crate) actions: &'a mut Vec<(Entity, ActionFunc)>,
	pub(crate) traversal: &'a mut Traversal,
	pub(crate) anim: &'a mut SpriteAnimation,
	pub(crate) ai: &'a mut MobjAi,
	pub(crate) pos: Position,
	pub(crate) cur_sector: CurrentSector,
	pub(crate) rot: MonsterRotation,
	pub(crate) mobj: MobjType,
}

pub(crate) fn look(ctx: &mut ActionContext, ent: Entity) {
	let mut query = ctx.world.query_one::<(
		&mut SpriteAnimation,
		&mut MobjAi,
		&Position,
		&CurrentSector,
		&MonsterRotation,
		&MobjType,
	)>(ent);

	let mut move_ctx = match query.get() {
		Ok((anim, ai, pos, cur_sector, rot, mobj)) => LookContext {
			world: ctx.world,
			ent,
			db: ctx.db,
			level: ctx.level,
			cmd: ctx.cmd,
			random: ctx.random,
			audio: ctx.audio,
			actions: ctx.actions,
			traversal: ctx.traversal,
			anim,
			ai,
			pos: *pos,
			cur_sector: *cur_sector,
			rot: *rot,
			mobj: *mobj,
		},
		Err(_) => return,
	};

	check_sound(&mut move_ctx, ctx.sound_targets);

	check_sight(&mut move_ctx);
}

fn check_sound(ctx: &mut LookContext, sound_targets: &mut [Option<Entity>]) {
	if let Some(sound_target_ent) = sound_targets[ctx.cur_sector.0.0] {
		if ctx.mobj.flags.contains(MobjFlags::AMBUSH) {
			let mut sound_target_query = ctx
				.world
				.query_one::<(&Position, &CurrentSector, &MobjType)>(sound_target_ent);
			let Ok((target_pos, target_sector, target)) =
				sound_target_query.get().map(|(p, s, t)| (*p, *s, *t))
			else {
				return;
			};

			if !p_check_sight(
				&SightContext {
					pos: ctx.pos,
					cur_sector: ctx.cur_sector,
					height: ctx.db.mobjinfo[&ctx.mobj.type_].height,
					target_pos,
					target_sector,
					target_height: ctx.db.mobjinfo[&target.type_].height,
					level: ctx.level,
				},
				ctx.traversal,
			) {
				return;
			}
		}

		wake_up_monster(ctx, sound_target_ent);
	}
}

fn check_sight(ctx: &mut LookContext) {
	let mut players_query = ctx
		.world
		.query::<(Entity, &Position, &CurrentSector, &MobjType)>()
		.with::<&PlayerMarker>();

	for (player_ent, player_pos, player_sector, player) in
		players_query.iter().map(|(_e, p, s, t)| (_e, *p, *s, *t))
	{
		if !player.flags.contains(MobjFlags::SHOOTABLE) {
			continue;
		}

		if !in_fov(ctx.pos, ctx.rot, player_pos) {
			continue;
		}

		if p_check_sight(
			&SightContext {
				pos: ctx.pos,
				cur_sector: ctx.cur_sector,
				height: ctx.db.mobjinfo[&ctx.mobj.type_].height,
				target_pos: player_pos,
				target_sector: player_sector,
				target_height: PLAYERHEIGHT,
				level: ctx.level,
			},
			ctx.traversal,
		) {
			wake_up_monster(ctx, player_ent);
		}
	}
}

fn wake_up_monster(ctx: &mut LookContext, target: Entity) {
	let mobj_info = ctx.db.mobjinfo.get(&ctx.mobj.type_).unwrap();

	if let (Some(see_state_num), Some(mut see_sound)) = (mobj_info.see_state, mobj_info.see_sound) {
		set_mobj_state(
			ctx.ent,
			ctx.ai,
			ctx.anim,
			see_state_num,
			ctx.actions,
			ctx.db,
			(ctx.random.p() & 0b111) as i32,
		);

		if see_sound.starts_with(b"DSPOSIT") {
			see_sound[7] = ctx.random.p() % 3 + b'1';
		} else if see_sound.starts_with(b"DSBGSIT") {
			see_sound[7] = ctx.random.p() % 2 + b'1';
		}

		let pos = if ctx.mobj.type_ == MobjNum::Spider || ctx.mobj.type_ == MobjNum::Cyborg {
			None
		} else {
			Some((ctx.pos.x, ctx.pos.y, ctx.pos.z))
		};

		ctx.audio.push(SfxEvent {
			sfx_id: to_u64(&see_sound),
			pos,
		})
	}

	ctx.cmd.insert(ctx.ent, (Target(target), Active));
}

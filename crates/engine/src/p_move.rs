use crate::{
	Active, CurrentSector, DB, Database, FLOATSPEED, InstantMoveIntent, MAXRADIUS, MobjFlagCommand,
	MobjFlags, MobjInfo, MobjNum, MobjType, MonsterRotation, Position, Random, Target, Velocity,
	WorldEvent, XSPEED, YSPEED, p_box_on_line_side, p_use_special_line,
};
use hecs::{Entity, World};
use rustc_hash::FxHashMap;
use wad_parser::{AABB, Level, LineFlags, LineId, SectorId};

pub(crate) struct MoveContext<'a> {
	pub(crate) ent: Entity,
	pub(crate) pos: &'a Position,
	pub(crate) goal_pos: (f32, f32, f32),
	pub(crate) mobj: &'a MobjType,
	pub(crate) mobj_info: &'a MobjInfo,
	pub(crate) imi: &'a mut InstantMoveIntent,
	pub(crate) level: &'a Level,
	pub(crate) world: &'a World,
	pub(crate) random: &'a mut Random,
	pub(crate) blocklists: &'a [Vec<Entity>],
	pub(crate) world_events: &'a mut Vec<WorldEvent>,
	pub(crate) inner: MoveContextInner,
}

#[derive(Default)]
pub(crate) struct MoveContextInner {
	ceilingline_idx: Option<LineId>,
	ceiling_y: f32,
	floor_y: f32,
	dropoff_y: f32,
	spec_hit: Vec<LineId>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn p_move(
	ctx: &mut MoveContext,
	rot: &MonsterRotation,
	mobj_flags: &mut Vec<MobjFlagCommand>,
) -> bool {
	let Some(move_dir) = rot.move_dir else {
		return false;
	};

	ctx.goal_pos.0 = ctx.pos.x + ctx.mobj_info.speed * XSPEED[move_dir as usize];
	ctx.goal_pos.2 = ctx.pos.z + ctx.mobj_info.speed * YSPEED[move_dir as usize];

	let (try_ok, float_ok) = p_try_move(ctx);

	if !try_ok {
		if ctx.mobj.flags.contains(MobjFlags::FLOAT) && float_ok {
			if ctx.pos.y < ctx.inner.floor_y {
				ctx.imi.dy += FLOATSPEED;
			} else {
				ctx.imi.dy -= FLOATSPEED;
			}

			mobj_flags.push(MobjFlagCommand::Add {
				ent: ctx.ent,
				flag: MobjFlags::IN_FLOAT,
			});
			return true;
		}

		if ctx.inner.spec_hit.is_empty() {
			return false;
		}

		let mut good = false;

		for line_id in ctx.inner.spec_hit.drain(..) {
			if p_use_special_line(ctx.mobj, &ctx.level.geom.lines[line_id.0]) {
				good = true;
			}
		}

		return good;
	} else {
		mobj_flags.push(MobjFlagCommand::Remove {
			ent: ctx.ent,
			flag: MobjFlags::IN_FLOAT,
		});
	}

	if !ctx.mobj.flags.contains(MobjFlags::FLOAT) {
		ctx.imi.dy = ctx.inner.floor_y - ctx.pos.y;
	}

	true
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn p_try_move(ctx: &mut MoveContext) -> (bool, bool) {
	// (try_ok, float_ok)

	let goal_sector_id = ctx.level.get_sector_by_pos(ctx.goal_pos.0, ctx.goal_pos.2);
	let goal_sector = &ctx.level.state.sectors[goal_sector_id.0];

	ctx.inner.ceiling_y = goal_sector.ceil_h;
	ctx.inner.floor_y = goal_sector.floor_h;
	ctx.inner.dropoff_y = goal_sector.floor_h;

	if !p_check_pos(ctx) {
		return (false, false);
	}

	if !ctx.mobj.flags.contains(MobjFlags::NO_CLIP) {
		if (ctx.inner.ceiling_y - ctx.inner.floor_y) < ctx.mobj_info.height {
			return (false, false);
		}

		if !ctx.mobj.flags.contains(MobjFlags::TELEPORT)
			&& ctx.inner.ceiling_y - ctx.pos.y < ctx.mobj_info.height
		{
			return (false, true);
		}

		if !ctx.mobj.flags.contains(MobjFlags::TELEPORT) && ctx.inner.floor_y - ctx.pos.y > 24.0 {
			return (false, true);
		}

		if !ctx
			.mobj
			.flags
			.intersects(MobjFlags::DROP_OFF | MobjFlags::FLOAT)
			&& ctx.inner.floor_y - ctx.inner.dropoff_y > 24.0
		{
			return (false, true);
		}
	}

	ctx.imi.dx = ctx.goal_pos.0 - ctx.pos.x;
	ctx.imi.dz = ctx.goal_pos.2 - ctx.pos.z;
	ctx.imi.new_sector = Some(ctx.level.get_sector_by_pos(ctx.goal_pos.0, ctx.goal_pos.2));

	(true, true)
}

#[allow(clippy::too_many_arguments)]
fn p_check_pos(ctx: &mut MoveContext) -> bool {
	if ctx.mobj.flags.contains(MobjFlags::NO_CLIP) {
		return true;
	}

	let bbox = AABB {
		min_x: ctx.goal_pos.0 - ctx.mobj_info.radius,
		max_x: ctx.goal_pos.0 + ctx.mobj_info.radius,
		min_z: ctx.goal_pos.2 - ctx.mobj_info.radius,
		max_z: ctx.goal_pos.2 + ctx.mobj_info.radius,
	};

	let (min_col, min_row) = ctx
		.level
		.geom
		.blockmap
		.world_to_grid(bbox.min_x - MAXRADIUS, bbox.min_z - MAXRADIUS);
	let (max_col, max_row) = ctx
		.level
		.geom
		.blockmap
		.world_to_grid(bbox.max_x + MAXRADIUS, bbox.max_z + MAXRADIUS);

	let db = DB.get().unwrap();

	for r in min_row..=max_row {
		for c in min_col..=max_col {
			let idx = r * ctx.level.geom.blockmap.col_num + c;

			for &other_entity in ctx.blocklists[idx].iter() {
				if other_entity == ctx.ent {
					continue;
				}

				let mut query = ctx
					.world
					.query_one::<(&MobjType, &Position, Option<&Target>)>(other_entity);

				if let Ok((other_type, other_pos, raw_target)) = query.get()
					&& !pit_check_thing(ctx, db, raw_target, other_entity, other_type, other_pos)
				{
					return false;
				}
			}

			for &line_idx in ctx.level.geom.blockmap.blocklists[idx].iter() {
				if !pit_check_line(ctx, bbox, line_idx) {
					return false;
				}
			}
		}
	}

	true
}

#[allow(clippy::too_many_arguments)]
fn pit_check_thing(
	ctx: &mut MoveContext,
	db: &Database,
	raw_target: Option<&Target>,
	other_ent: Entity,
	other_type: &MobjType,
	other_pos: &Position,
) -> bool {
	if !other_type
		.flags
		.intersects(MobjFlags::SOLID | MobjFlags::SPECIAL | MobjFlags::SHOOTABLE)
	{
		return true;
	}

	let mobj_info = &db.mobjinfo[&ctx.mobj.type_];
	let other_info = &db.mobjinfo[&other_type.type_];

	let blockdist = mobj_info.radius + other_info.radius;

	if (ctx.pos.z - other_pos.x).abs() >= blockdist || (ctx.pos.z - other_pos.z).abs() >= blockdist
	{
		return true;
	}

	if ctx.ent == other_ent {
		return true;
	}

	if ctx.mobj.flags.contains(MobjFlags::SKULL_FLY) {
		let damage = ((ctx.random.p() & 0b111) as u32 + 1) * mobj_info.damage;

		ctx.world_events.push(WorldEvent::DamageMobj {
			target: other_ent,
			inflictor: ctx.ent,
			damage,
		});

		ctx.world_events
			.push(WorldEvent::ResetSkullFly { actor_id: ctx.ent });

		return false;
	}

	if ctx.mobj.flags.contains(MobjFlags::MISSILE) {
		if ctx.pos.y > other_pos.y + other_info.height {
			return true;
		}
		if ctx.pos.y + mobj_info.height < other_pos.y {
			return true;
		}

		if let Some(target) = raw_target {
			let is_same_species = target.0 == other_ent
				|| ctx.mobj.type_ == other_type.type_
				|| (ctx.mobj.type_ == MobjNum::Knight && other_type.type_ == MobjNum::Bruiser)
				|| (ctx.mobj.type_ == MobjNum::Bruiser && other_type.type_ == MobjNum::Knight);

			if is_same_species {
				if other_ent == target.0 {
					return true;
				}

				if other_type.type_ != MobjNum::Player {
					return false;
				}
			}
		}

		if !other_type.flags.contains(MobjFlags::SHOOTABLE) {
			return !other_type.flags.contains(MobjFlags::SOLID);
		}

		let damage = ((ctx.random.p() % 0b111) as u32 + 1) * mobj_info.damage;
		ctx.world_events.push(WorldEvent::DamageMobj {
			target: other_ent,
			inflictor: ctx.ent,
			damage,
		});

		return false;
	}

	if other_type.flags.contains(MobjFlags::SPECIAL) {
		let solid = other_type.flags.contains(MobjFlags::SOLID);
		if ctx.mobj.flags.contains(MobjFlags::PICKUP) {
			ctx.world_events.push(WorldEvent::TouchSpecialThing {
				special_item: other_ent,
				picker: ctx.ent,
			});
		}
		return !solid;
	}

	!other_type.flags.contains(MobjFlags::SOLID)
}

fn pit_check_line(ctx: &mut MoveContext, bbox: AABB, line_id: LineId) -> bool {
	let line = &ctx.level.geom.lines[line_id.0];

	if !line.bbox.intersects_aabb(&bbox) {
		return true;
	}

	if p_box_on_line_side(&bbox, line, ctx.level) != -1 {
		return true;
	}

	if !ctx.mobj.flags.contains(MobjFlags::MISSILE) {
		if line.flags.contains(LineFlags::BLOCKING) {
			return false;
		}

		if ctx.mobj.type_ != MobjNum::Player && line.flags.contains(LineFlags::BLOCK_MONSTER) {
			return false;
		}
	}

	if let Some(open) = ctx.level.get_opening(line_id) {
		if open.top < ctx.inner.ceiling_y {
			ctx.inner.ceiling_y = open.top;
			ctx.inner.ceilingline_idx = Some(line_id);
		}

		if open.floor_high > ctx.inner.floor_y {
			ctx.inner.floor_y = open.floor_high;
		}

		if open.floor_low < ctx.inner.dropoff_y {
			ctx.inner.dropoff_y = open.floor_low;
		}

		if line.special != 0 && !ctx.level.state.lines[line_id.0].used {
			ctx.inner.spec_hit.push(line_id);
		}
	} else {
		return false;
	}

	true
}

type PendingMoves = FxHashMap<
	Entity,
	(
		f32,
		f32,
		f32,
		f32,
		f32,
		f32,
		usize,
		usize,
		usize,
		usize,
		Option<SectorId>,
	),
>;

/// Must be called after friction_system
pub fn try_move_system(
	world: &World,
	level: &Level,
	random: &mut Random,
	blocklists: &[Vec<Entity>],
	world_events: &mut Vec<WorldEvent>,
) -> PendingMoves {
	let db = DB.get().unwrap();
	let mut pending_moves = PendingMoves::default();

	let mut query = world
		.query::<(
			Entity,
			&mut InstantMoveIntent,
			&mut Velocity,
			&Position,
			&MobjType,
		)>()
		.with::<&Active>();

	for (ent, imi, velocity, pos, mobj) in query.iter() {
		let mut ctx = MoveContext {
			ent,
			pos,
			goal_pos: (pos.x + imi.dx, pos.y + imi.dy, pos.z + imi.dz),
			mobj,
			mobj_info: &db.mobjinfo[&mobj.type_],
			imi,
			level,
			world,
			random,
			blocklists,
			world_events,
			inner: MoveContextInner::default(),
		};
		let can_move = p_try_move(&mut ctx).0;

		if can_move {
			let (prev_col, prev_row) = level.geom.blockmap.world_to_grid(pos.x, pos.z);

			let prev_x = pos.x;
			let prev_y = pos.y;
			let prev_z = pos.z;

			let new_x = prev_x + imi.dx + velocity.x;
			let new_y = prev_y + imi.dy + velocity.y;
			let new_z = prev_z + imi.dz + velocity.z;

			let (new_col, new_row) = level.geom.blockmap.world_to_grid(new_x, new_z);

			pending_moves.insert(
				ent,
				(
					prev_x,
					prev_y,
					prev_z,
					new_x,
					new_y,
					new_z,
					prev_col,
					prev_row,
					new_col,
					new_row,
					imi.new_sector,
				),
			);
		} else {
			velocity.x = 0.0;
			velocity.y = 0.0;
			velocity.z = 0.0;
		}

		imi.dx = 0.0;
		imi.dy = 0.0;
		imi.dz = 0.0;

		imi.new_sector = None;
	}

	pending_moves
}

pub fn apply_monster_movement_system(
	world: &World,
	mut pending_moves: PendingMoves,
	level: &Level,
	blocklists: &mut [Vec<Entity>],
) {
	let mut query = world
		.query::<(Entity, &mut Position, &mut CurrentSector)>()
		.with::<&Active>();
	for (ent, pos, current_sector) in query.iter() {
		let Some((
			prev_x,
			prev_y,
			prev_z,
			new_x,
			new_y,
			new_z,
			prev_col,
			prev_row,
			new_col,
			new_row,
			new_sector,
		)) = pending_moves.remove(&ent)
		else {
			continue;
		};

		pos.prev_x = prev_x;
		pos.prev_y = prev_y;
		pos.prev_z = prev_z;

		pos.x = new_x;
		pos.y = new_y;
		pos.z = new_z;

		if prev_col != new_col || prev_row != new_row {
			let prev_idx = prev_row * level.geom.blockmap.col_num + prev_col;
			let new_idx = new_row * level.geom.blockmap.col_num + new_col;

			blocklists[prev_idx].retain(|&e| e != ent);
			blocklists[new_idx].push(ent);
		}

		if let Some(s) = new_sector {
			current_sector.0 = s;
		}
	}
}

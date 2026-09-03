use hecs::Entity;
use strum::IntoEnumIterator;

use crate::{
	DIAGS, Direction, MELEERANGE, MobjFlagCommand, MobjFlags, MobjNum, MobjType, MonsterRotation,
	MoveContext, OPPOSITE, Position, Random, SightContext, Traversal, fast_atan2, p_check_sight,
	p_move,
};

pub fn aprox_xz_distance(src: (f32, f32), dst: (f32, f32)) -> f32 {
	let dx = (src.0 - dst.0).abs();
	let dz = (src.1 - dst.1).abs();

	dx + dz - dx.min(dz) * 0.5
}

pub fn aprox_xyz_distance(src: (f32, f32, f32), dst: (f32, f32, f32)) -> f32 {
	aprox_xz_distance((src.0, src.2), (dst.0, dst.2)) + (src.1 - dst.1).abs() * 0.5
}

pub fn in_fov(pos: Position, rot: MonsterRotation, player_pos: Position) -> bool {
	if rot.move_dir.is_none() {
		return false;
	}

	let move_dir = rot.move_dir.unwrap() as u32;
	let angle_to_player = fast_atan2(player_pos.x - pos.x, player_pos.z - pos.z) >> 29;

	// after the division above, miracles of rounding happen
	angle_to_player == (move_dir.wrapping_sub(2) & 0b111)
		|| angle_to_player == (move_dir.wrapping_sub(1) & 0b111)
		|| angle_to_player == move_dir
		|| angle_to_player == ((move_dir + 1) & 0b111)
}

pub(crate) fn p_check_melee_range(
	ctx: &SightContext,
	traversal: &mut Traversal,
	target_radius: f32,
) -> bool {
	if !p_check_sight(ctx, traversal) {
		return false;
	}

	let dist = aprox_xz_distance((ctx.target_pos.x, ctx.target_pos.z), (ctx.pos.x, ctx.pos.z));

	dist < MELEERANGE - 20.0 + target_radius
}

pub(crate) fn p_check_missile_range(
	ctx: &SightContext,
	ent: Entity,
	mobj: MobjType,
	traversal: &mut Traversal,
	random: &mut Random,
	mobj_flags: &mut Vec<MobjFlagCommand>,
	is_melee_state_none: bool,
) -> bool {
	if !p_check_sight(ctx, traversal) {
		return false;
	}

	if mobj.flags.contains(MobjFlags::JUST_HIT) {
		mobj_flags.push(MobjFlagCommand::Remove {
			ent,
			flag: MobjFlags::JUST_HIT,
		});
		return true;
	}

	let mut dist =
		aprox_xz_distance((ctx.target_pos.x, ctx.target_pos.z), (ctx.pos.x, ctx.pos.z)) - 64.0;

	if is_melee_state_none {
		dist -= 128.0;
	}

	match mobj.type_ {
		MobjNum::Vile => {
			if dist > 14.0 * 64.0 {
				return false;
			}
		}

		MobjNum::Undead => {
			if dist < 196.0 {
				return false;
			} else {
				dist /= 2.0;
			}
		}

		MobjNum::Cyborg | MobjNum::Spider | MobjNum::Skull => {
			dist /= 2.0;
		}

		_ => {}
	}

	if dist > 200.0 {
		dist = 200.0;
	}

	if mobj.type_ == MobjNum::Cyborg && dist > 160.0 {
		dist = 160.0;
	}

	(random.p() as f32) >= dist
}

pub(crate) fn p_new_chase_dir(
	ctx: &mut MoveContext,
	rot: &mut MonsterRotation,
	target_pos: Position,
	mobj_flags: &mut Vec<MobjFlagCommand>,
) {
	let old_dir = rot.move_dir;
	let turnaround = old_dir.map(|dir| OPPOSITE[dir as usize]);

	let dx = target_pos.x - ctx.pos.x;
	let dz = target_pos.z - ctx.pos.z;

	let mut dir: [Option<Direction>; 3] = [None; 3];

	if dx > 10.0 {
		dir[1] = Some(Direction::East);
	} else if dx < -10.0 {
		dir[1] = Some(Direction::West);
	} else {
		dir[1] = None;
	}

	if dz < -10.0 {
		dir[2] = Some(Direction::South);
	} else if dz > 10.0 {
		dir[2] = Some(Direction::North);
	} else {
		dir[2] = None;
	}

	// because of borrow checker
	let random1 = ctx.random.p();
	let random2 = ctx.random.p();

	let mut try_walk = |dir: Option<Direction>| -> bool {
		rot.move_dir = dir;
		p_move(ctx, rot, mobj_flags)
	};

	if dir[1].is_some() && dir[2].is_some() {
		let idx = (((dz < 0.0) as usize) << 1) + ((dx > 0.0) as usize);
		let diag_dir = Some(DIAGS[idx]);

		if diag_dir != turnaround && try_walk(diag_dir) {
			return;
		}
	}

	if random1 > 200 || dz.abs() > dx.abs() {
		dir.swap(1, 2);
	}

	if dir[1] == turnaround {
		dir[1] = None;
	}
	if dir[2] == turnaround {
		dir[2] = None;
	}

	if try_walk(dir[1]) {
		return;
	}

	if try_walk(dir[2]) {
		return;
	}

	if try_walk(old_dir) {
		return;
	}

	if random2 & 0b1 != 0 {
		for tdir in Direction::iter() {
			if Some(tdir) != turnaround && try_walk(Some(tdir)) {
				return;
			}
		}
	} else {
		for tdir in Direction::iter().rev() {
			if Some(tdir) != turnaround && try_walk(Some(tdir)) {
				return;
			}
		}
	}

	if try_walk(turnaround) {
		return;
	}

	rot.move_dir = None;
}

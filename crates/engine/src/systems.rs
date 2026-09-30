use crate::{
	Action, Active, Collider, DB, FRICTION, MobjAi, MobjFlags, MobjType, Position, SpriteAnimation,
	StateNum, Target, Velocity,
};
use hecs::{Entity, World};
use wad_parser::Level;

pub enum MobjFlagCommand {
	Remove { ent: Entity, flag: MobjFlags },
	Add { ent: Entity, flag: MobjFlags },
}

pub fn apply_mobj_flags_system(mobj_flags: &mut Vec<MobjFlagCommand>, world: &mut World) {
	for command in mobj_flags.drain(..) {
		match command {
			MobjFlagCommand::Remove { ent, flag } => {
				world
					.query_one_mut::<&mut MobjType>(ent)
					.unwrap()
					.flags
					.remove(flag);
			}
			MobjFlagCommand::Add { ent, flag } => {
				world
					.query_one_mut::<&mut MobjType>(ent)
					.unwrap()
					.flags
					.insert(flag);
			}
		}
	}
}

pub fn update_blocklists_system(
	world: &mut World,
	level: &Level,
	blocklists: &mut [Vec<(Entity, Collider)>],
) {
	world
		.query_mut::<(Entity, &Position, &MobjType, Option<&Target>)>()
		.with::<&Active>()
		.into_iter()
		.map(|(_e, p, m, t)| (_e, *p, *m, t.copied()))
		.for_each(|(ent, pos, mobj, target)| {
			let (col, row) = level.geom.blockmap.world_to_grid(pos.x, pos.z);
			let idx = row * level.geom.blockmap.col_num + col;

			for i in 0..blocklists[idx].len() {
				let (e, coll) = &mut blocklists[idx][i];
				if *e == ent {
					*coll = Collider { pos, mobj, target };
					break;
				}
			}
		});
}

/// Must be called after handle_position_input
pub fn friction_system(world: &mut World) {
	for velocity in world.query_mut::<&mut Velocity>() {
		velocity.x *= FRICTION;
		velocity.y *= 0.7;
		velocity.z *= FRICTION;
	}
}

pub fn animation_system(world: &mut World) {
	let db = DB.get().unwrap();
	for (anim, ai) in world.query_mut::<(&mut SpriteAnimation, &MobjAi)>() {
		let state_data = db.states[ai.current_state as usize];
		anim.cached_rotations = state_data.cached_rotations;
		anim.full_bright = state_data.frame & (1 << 15) != 0;
	}
}

#[derive(Debug, Clone, Copy)]
pub struct StateCommand {
	pub ent: Entity,
	pub state: StateNum,
	pub tics_to_add: i32,
}

pub fn state_system(world: &mut World, state_buffer: &mut Vec<StateCommand>) {
	let db = DB.get().unwrap();

	for StateCommand {
		ent,
		state,
		tics_to_add,
	} in state_buffer.drain(..)
	{
		let Ok((ai, act, anim)) =
			world.query_one_mut::<(&mut MobjAi, &mut Action, &mut SpriteAnimation)>(ent)
		else {
			continue;
		};

		ai.current_state = state;

		let state = db.states[state as usize];
		ai.tics_left = state.tics + tics_to_add;
		anim.cached_rotations = state.cached_rotations;
		act.0 = state.action;
	}
}

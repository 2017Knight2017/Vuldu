use crate::{DB, FRICTION, MobjAi, MobjFlags, MobjType, SpriteAnimation, Velocity};
use hecs::{Entity, World};

pub enum MobjFlagCommand {
	Remove { ent: Entity, flag: MobjFlags },
	Add { ent: Entity, flag: MobjFlags },
}

pub fn apply_mobj_flags_system(mobj_flag_buffer: &mut Vec<MobjFlagCommand>, world: &World) {
	for command in mobj_flag_buffer.drain(..) {
		match command {
			MobjFlagCommand::Remove { ent, flag } => {
				world.get::<&mut MobjType>(ent).unwrap().flags.remove(flag);
			}
			MobjFlagCommand::Add { ent, flag } => {
				world.get::<&mut MobjType>(ent).unwrap().flags.insert(flag);
			}
		}
	}
}

/// Must be called after handle_position_input
pub fn friction_system(world: &World) {
	let mut query = world.query::<&mut Velocity>();
	for velocity in query.iter() {
		velocity.x *= FRICTION;
		velocity.y *= 0.7;
		velocity.z *= FRICTION;
	}
}

pub fn animation_system(world: &World) {
	let mut query = world.query::<(&mut SpriteAnimation, &MobjAi)>();
	for (anim, ai) in query.iter() {
		let db = DB.get().unwrap();
		anim.cached_rotations = db.states[&ai.current_state].cached_rotations;
	}
}

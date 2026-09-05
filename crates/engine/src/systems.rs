use crate::{
	Active, Collider, DB, FRICTION, MobjAi, MobjFlags, MobjType, Position, SpriteAnimation, Target,
	Velocity,
};
use hecs::{Entity, World};
use rustc_hash::FxHashMap;
use wad_parser::Level;

pub enum MobjFlagCommand {
	Remove { ent: Entity, flag: MobjFlags },
	Add { ent: Entity, flag: MobjFlags },
}

pub fn apply_mobj_flags_system(mobj_flags: &mut Vec<MobjFlagCommand>, world: &World) {
	for command in mobj_flags.drain(..) {
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

pub fn update_blocklists_system(
	world: &World,
	level: &Level,
	blocklists: &mut [FxHashMap<Entity, Collider>],
) {
	world
		.query::<(Entity, &Position, &MobjType, Option<&Target>)>()
		.with::<&Active>()
		.iter()
		.map(|(_e, p, m, t)| (_e, *p, *m, t.copied()))
		.for_each(|(ent, pos, mobj, target)| {
			let (col, row) = level.geom.blockmap.world_to_grid(pos.x, pos.z);
			let idx = row * level.geom.blockmap.col_num + col;
			*blocklists[idx].get_mut(&ent).unwrap() = Collider { pos, mobj, target };
		});
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
	let db = DB.get().unwrap();
	let mut query = world.query::<(&mut SpriteAnimation, &MobjAi)>();
	for (anim, ai) in query.iter() {
		let state_data = db.states[&ai.current_state];
		anim.cached_rotations = state_data.cached_rotations;
		anim.full_bright = state_data.frame & (1 << 15) != 0;
	}
}

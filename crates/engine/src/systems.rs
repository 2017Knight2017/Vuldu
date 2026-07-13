use crate::{
	components::{Velocity, Transform, SpriteAnimation},
	constants::FRICTION,
    data_tables::DB,
};
use hecs::World;

pub fn system_movement_and_friction(world: &mut World) {
	for (transform, velocity) in world.query_mut::<(&mut Transform, &mut Velocity)>() {
		velocity.x *= FRICTION;
		velocity.y *= 0.7;
		velocity.z *= FRICTION;

		transform.prev_x = transform.x;
        transform.prev_y = transform.y;
		transform.prev_z = transform.z;
        transform.x += velocity.x;
        transform.y += velocity.y;
        transform.z += velocity.z;
    }
}

pub fn animation_system(world: &mut World) {
    for anim in world.query_mut::<&mut SpriteAnimation>() {
        if anim.tics_left <= 0 {
            continue; 
        }

        anim.tics_left -= 1;
        if anim.tics_left != 0 { 
            continue; 
        }

        if let (Some(db), Some(current_state)) = (DB.get(), anim.current_state) {
            let current_state_data = db.states[&current_state];

            if let Some(next_state_id) = current_state_data.next_state {
                let next_state_data = db.states[&next_state_id];
                
                anim.current_state = Some(next_state_id);
                anim.tics_left = next_state_data.tics;
                anim.cached_rotations = next_state_data.cached_rotations;
            }
        }
    }
}

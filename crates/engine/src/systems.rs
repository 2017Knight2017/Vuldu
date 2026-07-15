use crate::{
	components::{Velocity, Position, SpriteAnimation},
	constants::FRICTION,
    data_tables::DB,
};
use hecs::QueryBorrow;

pub fn movement_system(mut query: QueryBorrow<'_, (&mut Position, &Velocity)>) {
	for (position, velocity) in query.iter() {
		position.prev_x = position.x;
        position.prev_y = position.y;
		position.prev_z = position.z;
        position.x += velocity.x;
        position.y += velocity.y;
        position.z += velocity.z;
    }
}

pub fn friction_system(mut query: QueryBorrow<'_, &mut Velocity>) {
    for velocity in query.iter() {
		velocity.x *= FRICTION;
		velocity.y *= 0.7;
		velocity.z *= FRICTION;
    }
}

pub fn animation_system(mut query: QueryBorrow<'_, &mut SpriteAnimation>) {
    for anim in query.iter() {
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

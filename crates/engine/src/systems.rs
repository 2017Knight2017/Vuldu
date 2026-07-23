use crate::{Active, CurrentSector, DB, FRICTION, Position, SpriteAnimation, Velocity};
use wad_parser::DoomMap;
use hecs::QueryBorrow;

pub fn movement_system(mut query: QueryBorrow<'_, (&mut Position, &mut CurrentSector, &Velocity, &Active)>, map: &DoomMap) {
    //! Must be called after friction_system
    for (position, current_sector, velocity, _active) in query.iter() {
		position.prev_x = position.x;
        position.prev_y = position.y;
		position.prev_z = position.z;
        position.x += velocity.x;
        position.y += velocity.y;
        position.z += velocity.z;

        current_sector.0 = map.get_sector_by_pos(-position.x, position.z);
    }
}

pub fn friction_system(mut query: QueryBorrow<'_, &mut Velocity>) {
    //! Must be called after handle_position_input
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

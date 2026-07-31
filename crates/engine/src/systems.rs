use crate::{Active, CurrentSector, DB, FRICTION, InstantMoveIntent, MobjAi, MobjType, PlayerMarker, Position, Random, SpriteAnimation, Velocity, WorldEvent, p_try_move};
use wad_parser::DoomMap;
use hecs::{Entity, QueryBorrow, With, World};

/// Must be called after friction_system
pub fn monster_movement_system(
    mut query: QueryBorrow<'_, With<(
        Entity, 
        &mut Position, 
        &mut CurrentSector, 
        &mut InstantMoveIntent, 
        &mut Velocity,
        &MobjType,
    ), &Active>>, 
    map: &DoomMap,
    world: &World,
    random: &mut Random,
    blocklists: &mut [Vec<Entity>],
    world_events: &mut Vec<WorldEvent>
) {
    let db = DB.get().unwrap();
    for (ent, pos, current_sector, imi, velocity, mobj_type) in query.iter() {
        let can_move = p_try_move(
            ent, 
            pos, 
            (pos.x + imi.dx, pos.y + imi.dy, pos.z + imi.dy), 
            mobj_type, 
            &db.mobjinfo[&mobj_type.type_], 
            imi.new_sector.unwrap_or(current_sector.0), 
            imi, 
            map, 
            world, 
            random, 
            blocklists, 
            world_events
        );

        if can_move.0 {
            let (prev_col, prev_row) = map.blockmap.world_to_grid(pos.x, pos.z);

            pos.x += imi.dx;
            pos.y += imi.dy;
            pos.z += imi.dz;

            pos.prev_x = pos.x;
            pos.prev_y = pos.y;
		    pos.prev_z = pos.z;

            pos.x += velocity.x;
            pos.y += velocity.y;
            pos.z += velocity.z;

            let (col, row) = map.blockmap.world_to_grid(pos.x, pos.z);

            if prev_col != col || prev_row != row {
                blocklists[prev_row * map.blockmap.col_num + prev_col].retain(|&e| e != ent);
                blocklists[col * map.blockmap.col_num + row].push(ent);
            }

            if let Some(new_sector) = imi.new_sector {
                current_sector.0 = new_sector;
            }

            imi.new_sector = None;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
            velocity.z = 0.0;

            imi.dx = 0.0;
            imi.dy = 0.0;
            imi.dz = 0.0;
        }
    }
}

pub fn player_movement_system(
    mut query: QueryBorrow<'_, With<(&mut Position, &Velocity, &mut CurrentSector), &PlayerMarker>>,
    map: &DoomMap
) {
    for (pos, velocity, current_sector) in query.iter() {
        pos.prev_x = pos.x;
        pos.prev_y = pos.y;
	    pos.prev_z = pos.z;
        pos.x += velocity.x;
        pos.y += velocity.y;
        pos.z += velocity.z;

        current_sector.0 = map.get_sector_by_pos(pos.x, pos.z);
    }
}

/// Must be called after handle_position_input
pub fn friction_system(mut query: QueryBorrow<'_, &mut Velocity>) { 
    for velocity in query.iter() {
		velocity.x *= FRICTION;
		velocity.y *= 0.7;
		velocity.z *= FRICTION;
    }
}

pub fn animation_system(mut query: QueryBorrow<'_, (&mut SpriteAnimation, &MobjAi)>) {
    for (anim, ai) in query.iter() {
        let db = DB.get().unwrap();
        anim.cached_rotations = db.states[&ai.current_state].cached_rotations;  
    }
}

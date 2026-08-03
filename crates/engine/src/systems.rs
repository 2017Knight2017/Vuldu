use crate::{Active, CurrentSector, DB, FRICTION, InstantMoveIntent, MobjAi, MobjFlags, MobjType, PlayerMarker, Position, Random, SpriteAnimation, Velocity, WorldEvent, p_try_move};
use rustc_hash::FxHashMap;
use wad_parser::DoomMap;
use hecs::{Entity, QueryBorrow, With, World};

type PendingMoves = 
    FxHashMap<Entity, (f32, f32, f32, f32, f32, f32, usize, usize, usize, usize, Option<usize>)>;

/// Must be called after friction_system
pub fn try_move_system(
    mut try_move_query: QueryBorrow<'_, With<(
        Entity, 
        &mut InstantMoveIntent, 
        &mut Velocity,
        &Position, 
        &MobjType,
    ), &Active>>, 
    map: &DoomMap,
    world: &World,
    random: &mut Random,
    blocklists: &mut [Vec<Entity>],
    world_events: &mut Vec<WorldEvent>
) -> PendingMoves {
    let db = DB.get().unwrap();
    let mut pending_moves = PendingMoves::default();

    for (ent, imi, velocity, pos, mobj_type) in try_move_query.iter() {
        let can_move = p_try_move(
            ent, 
            pos, 
            (pos.x + imi.dx, pos.y + imi.dy, pos.z + imi.dz), 
            mobj_type, 
            &db.mobjinfo[&mobj_type.type_], 
            imi, 
            map, 
            world, 
            random, 
            blocklists, 
            world_events
        ).0;

        if can_move {
            let (prev_col, prev_row) = map.blockmap.world_to_grid(pos.x, pos.z);

            let prev_x = pos.x + imi.dx;
            let prev_y = pos.y + imi.dy;
            let prev_z = pos.z + imi.dz;

            let new_x = prev_x + velocity.x;
            let new_y = prev_y + velocity.y;
            let new_z = prev_z + velocity.z;

            let (new_col, new_row) = map.blockmap.world_to_grid(new_x, new_z);

            pending_moves.insert(ent, (
                prev_x, prev_y, prev_z,
                new_x, new_y, new_z,
                prev_col, prev_row,
                new_col, new_row,
                imi.new_sector
            ));
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
            velocity.z = 0.0;

            imi.dx = 0.0;
            imi.dy = 0.0;
            imi.dz = 0.0;
        }

        imi.new_sector = None;
    }

    pending_moves
}

pub fn apply_monster_movement_system(
    mut apply_query: QueryBorrow<'_, With<(
        Entity, 
        &mut Position,
        &mut CurrentSector, 
    ), &Active>>,
    pending_moves: PendingMoves,
    map: &DoomMap,
    blocklists: &mut [Vec<Entity>],
) {
    for (ent, pos, current_sector) in apply_query.iter() {
        let (prev_x, prev_y, prev_z,
            new_x, new_y, new_z,
            prev_col, prev_row,
            new_col, new_row,
            new_sector) = match pending_moves.get(&ent) 
        {
            Some(data) => *data,
            None => continue 
        };

        pos.prev_x = prev_x;
        pos.prev_y = prev_y;
        pos.prev_z = prev_z;

        pos.x = new_x;
        pos.y = new_y;
        pos.z = new_z;

        if prev_col != new_col || prev_row != new_row {
            let prev_idx = prev_row * map.blockmap.col_num + prev_col;
            let new_idx = new_row * map.blockmap.col_num + new_col;

            blocklists[prev_idx].retain(|&e| e != ent);
            blocklists[new_idx].push(ent);
        }

        if let Some(s) = new_sector {
            current_sector.0 = s;  
        }
    }
}

pub enum MobjFlagCommand {
    Remove { ent: Entity, flag: MobjFlags },
    Add { ent: Entity, flag: MobjFlags } 
}

pub fn apply_mobj_flags_system(
    mobj_flag_buffer: &mut Vec<MobjFlagCommand>,
    world: &World
) {
    for command in mobj_flag_buffer.drain(..) {
        match command {
            MobjFlagCommand::Remove { ent, flag } => {
                world.get::<&mut MobjType>(ent)
                    .unwrap()
                    .flags
                    .remove(flag);
            },
            MobjFlagCommand::Add { ent, flag } => {
                world.get::<&mut MobjType>(ent)
                    .unwrap()
                    .flags
                    .insert(flag);
            } 
        }
    }
}

pub fn apply_player_movement_system(
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

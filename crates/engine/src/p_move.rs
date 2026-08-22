use hecs::{Entity, World};
use rustc_hash::FxHashMap;
use wad_parser::{AABB, Level, LineFlags, LineId, SectorId};
use crate::{Active, CurrentSector, DB, FLOATSPEED, InstantMoveIntent, MAXRADIUS, MAXSPECHIT, MobjFlagCommand, MobjFlags, MobjInfo, MobjNum, MobjType, MonsterRotation, Position, Random, Target, Velocity, WorldEvent, XSPEED, YSPEED, p_box_on_line_side};

#[derive(Debug, Clone)]
struct MoveContext {
    ceilingline_idx: Option<LineId>, 
    ceiling_y: f32, 
    floor_y: f32, 
    dropoff_y: f32, 
    spec_hit: Vec<LineId>,
}

pub fn p_move(
    ent: Entity,
	pos: &Position,
	rot: &MonsterRotation,
    mobj_type: &MobjType,
    mobj_info: &MobjInfo,
	imi: &mut InstantMoveIntent,
    level: &Level,
    world: &World,
    random: &mut Random,
    blocklists: &[Vec<Entity>], 
    world_events: &mut Vec<WorldEvent>,
    mobj_flag_buffer: &mut Vec<MobjFlagCommand>
) -> bool {
    if rot.move_dir.is_none() {
        return false;
    }

    let move_dir = rot.move_dir.unwrap();

    let try_x = pos.x + mobj_info.speed * 0.41 * XSPEED[move_dir as usize];
    let try_z = pos.z + mobj_info.speed * 0.41 * YSPEED[move_dir as usize];

    let (try_ok, float_ok) = p_try_move(
        ent,
        pos, 
        (try_x, pos.y, try_z), 
        mobj_type, 
        mobj_info, 
        imi, 
        level,
        world,
        random, 
        blocklists,
        world_events
    );
    
    if let Some(new_sector) = imi.new_sector {
        if !try_ok {
            if mobj_type.flags.contains(MobjFlags::FLOAT) && float_ok {
                if pos.y < level.state.sectors[new_sector.0].floor_h {
                    imi.dy += FLOATSPEED;
                } else {
                    imi.dy -= FLOATSPEED;
                }

                mobj_flag_buffer.push(MobjFlagCommand::Add { ent, flag: MobjFlags::IN_FLOAT });
                return true;
            }

            //if ctx.spec_hit.is_empty() {
            //    return false;
            //}
            //
            //let mut good = false;
            //
            //for line_idx in ctx.spec_hit.drain(..) {
            //    if p_use_special_line(actor, line_idx, 0, ctx) {
            //        good = true;
            //    }
            //}
            //
            //return good;
        } else {
            mobj_flag_buffer.push(MobjFlagCommand::Remove { ent, flag: MobjFlags::IN_FLOAT });
        }

        if !mobj_type.flags.contains(MobjFlags::FLOAT) {
            imi.dy = level.state.sectors[new_sector.0].floor_h - pos.y;
        }
    }

    true
}

pub fn p_try_move(
    ent: Entity,
    pos: &Position,
    goal_pos: (f32, f32, f32),
	mobj_type: &MobjType,
	mobj_info: &MobjInfo,
    imi: &mut InstantMoveIntent,
	map: &Level, 
    world: &World,
    random: &mut Random,
    blocklists: &[Vec<Entity>],
    world_events: &mut Vec<WorldEvent>
) -> (bool, bool) {
	// (try_ok, float_ok)

    let mut ctx = MoveContext { 
        ceilingline_idx: None, 
        ceiling_y: f32::MAX, 
        floor_y: f32::MIN, 
        dropoff_y: f32::MAX, 
        spec_hit: Vec::with_capacity(MAXSPECHIT) 
    };

	if !p_check_pos(
        &mut ctx,
        ent,
        mobj_type,
        goal_pos,
        mobj_info,
        map,
        world,
        random,
        blocklists,
        world_events
    ) {
		return (false, false)
	}

	if !mobj_type.flags.contains(MobjFlags::NO_CLIP) {
		if ((ctx.ceiling_y - ctx.floor_y) as f32) < mobj_info.height {
            return (false, false);
        }

        if !mobj_type.flags.contains(MobjFlags::TELEPORT) &&
            ctx.ceiling_y - pos.y < mobj_info.height
        {
            return (false, true);
        }

        if !mobj_type.flags.contains(MobjFlags::TELEPORT) &&
            ctx.floor_y - pos.y > 24.0 
        {
            return (false, true);
        }

        if !mobj_type.flags.intersects(MobjFlags::DROP_OFF | MobjFlags::FLOAT) &&
            ctx.floor_y - ctx.dropoff_y > 24.0
        {
            return (false, true);
        }
	}

    imi.dx = goal_pos.0 - pos.x;
    imi.dz = goal_pos.2 - pos.z;
    imi.new_sector = Some(map.get_sector_by_pos(goal_pos.0, goal_pos.2));

    (true, true)
}

fn p_check_pos(
    ctx: &mut MoveContext,
    ent: Entity,
    mobj_type: &MobjType,
    goal_pos: (f32, f32, f32),
    mobj_info: &MobjInfo,
    level: &Level,
    world: &World,
    random: &mut Random,
    blocklists: &[Vec<Entity>],
    world_events: &mut Vec<WorldEvent>,
) -> bool {
    if mobj_type.flags.contains(MobjFlags::NO_CLIP) {
        return true;
    }

    let bbox = AABB {
        min_x: goal_pos.0 - mobj_info.radius,
        max_x: goal_pos.0 + mobj_info.radius,
        min_z: goal_pos.2 - mobj_info.radius,
        max_z: goal_pos.2 + mobj_info.radius,
    };

	let (min_col, min_row) = level.geom.blockmap.world_to_grid(bbox.min_x - MAXRADIUS, bbox.min_z - MAXRADIUS);
    let (max_col, max_row) = level.geom.blockmap.world_to_grid(bbox.max_x + MAXRADIUS, bbox.max_z + MAXRADIUS);

    for r in min_row..=max_row {
        for c in min_col..=max_col {
            let idx = r * level.geom.blockmap.col_num + c;

            for &other_entity in blocklists[idx].iter() {
                if other_entity == ent {
                    continue;
                }

                let mut query = world.query_one::<(&MobjType, &Position, Option<&Target>)>(other_entity);
                if let Ok((other_type, other_pos, raw_target)) = query.get() {
                    if !pit_check_thing(ent, mobj_type, goal_pos, raw_target, other_entity, other_type, other_pos, random, world_events) {
                        return false;
                    }
                }
            }

            for &line_idx in level.geom.blockmap.blocklists[idx].iter() {
                if !pit_check_line(ctx, mobj_type, bbox, line_idx, level) {
                    return false;
                }
            }
        }
    }

    true
}

fn pit_check_thing(
	ent: Entity,
	mobj_type: &MobjType,
	pos: (f32, f32, f32),
	raw_target: Option<&Target>,
    other_ent: Entity,
	other_type: &MobjType,
	other_pos: &Position,
    random: &mut Random,
    world_events: &mut Vec<WorldEvent>,
) -> bool {
    let db = DB.get().unwrap();

    if !other_type.flags.intersects(
        MobjFlags::SOLID | MobjFlags::SPECIAL | MobjFlags::SHOOTABLE,
    ) {
        return true;
    }

	let mobj_info = &db.mobjinfo[&mobj_type.type_];
	let other_info = &db.mobjinfo[&other_type.type_];

    let blockdist = mobj_info.radius + other_info.radius;

    if (pos.0 - other_pos.x).abs() >= blockdist || (pos.2 - other_pos.z).abs() >= blockdist {
        return true;
    }

    if ent == other_ent {
        return true;
    }

    if mobj_type.flags.contains(MobjFlags::SKULL_FLY) {
        let damage = ((random.p() % 8) as u32 + 1) * mobj_info.damage;

        world_events.push(WorldEvent::DamageMobj {
            target: other_ent,
            inflictor: ent,
            damage,
        });

        world_events.push(WorldEvent::ResetSkullFly {
            actor_id: ent,
        });

        return false;
    }

    if mobj_type.flags.contains(MobjFlags::MISSILE) {
        if pos.1 > other_pos.y + other_info.height {
            return true;
        }
        if pos.1 + mobj_info.height < other_pos.y {
            return true;
        }

        if let Some(target) = raw_target {
            let is_same_species = target.0 == other_ent
                || mobj_type.type_ == other_type.type_
                || (mobj_type.type_ == MobjNum::Knight && other_type.type_ == MobjNum::Bruiser)
                || (mobj_type.type_ == MobjNum::Bruiser && other_type.type_ == MobjNum::Knight);

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

        let damage = ((random.p() % 8) as u32 + 1) * mobj_info.damage;
        world_events.push(WorldEvent::DamageMobj {
            target: other_ent,
            inflictor: ent,
            damage,
        });

        return false;
    }

    if other_type.flags.contains(MobjFlags::SPECIAL) {
        let solid = other_type.flags.contains(MobjFlags::SOLID);
        if mobj_type.flags.contains(MobjFlags::PICKUP) {
            world_events.push(WorldEvent::TouchSpecialThing {
                special_item: other_ent,
                picker: ent,
            });
        }
        return !solid;
    }

    !other_type.flags.contains(MobjFlags::SOLID)
}

fn pit_check_line(
    ctx: &mut MoveContext,
    mobj_type: &MobjType,
    bbox: AABB,
    line_id: LineId,
    level: &Level,
) -> bool {
    let line = &level.geom.lines[line_id.0];

    if !line.bbox.intersects_aabb(&bbox) {
        return true;
    }

    if p_box_on_line_side(&bbox, &line, level) != -1 {
        return true;
    }

    if !mobj_type.flags.contains(MobjFlags::MISSILE) {
        if line.flags.contains(LineFlags::BLOCKING) {
            return false;
        }

        if mobj_type.type_ != MobjNum::Player && line.flags.contains(LineFlags::BLOCK_MONSTER) {
            return false;
        }
    }

	if let Some(open) = level.get_opening(line_id) {
        if open.top < ctx.ceiling_y {
            ctx.ceiling_y = open.top;
            ctx.ceilingline_idx = Some(line_id);
        }

        if open.floor_high > ctx.floor_y {
            ctx.floor_y = open.floor_high;
        }

        if open.floor_low < ctx.dropoff_y {
            ctx.dropoff_y = open.floor_low;
        }

        if line.special != 0 {
            if ctx.spec_hit.len() < 8 {
                ctx.spec_hit.push(line_id);
            }
        }
    } else {
        return false; 
    }

    true
}

type PendingMoves = 
    FxHashMap<Entity, (f32, f32, f32, f32, f32, f32, usize, usize, usize, usize, Option<SectorId>)>;

/// Must be called after friction_system
pub fn try_move_system(
    world: &World,
    level: &Level,
    random: &mut Random,
    blocklists: &[Vec<Entity>],
    world_events: &mut Vec<WorldEvent>
) -> PendingMoves {
    let db = DB.get().unwrap();
    let mut pending_moves = PendingMoves::default();

    let mut query = world
        .query::<(Entity, &mut InstantMoveIntent, &mut Velocity, &Position, &MobjType)>()
        .with::<&Active>();
    for (ent, imi, velocity, pos, mobj_type) in query.iter() {
        let can_move = p_try_move(
            ent, 
            pos, 
            (pos.x + imi.dx, pos.y + imi.dy, pos.z + imi.dz), 
            mobj_type, 
            &db.mobjinfo[&mobj_type.type_], 
            imi, 
            level, 
            world, 
            random, 
            blocklists, 
            world_events
        ).0;

        if can_move {
            let (prev_col, prev_row) = level.geom.blockmap.world_to_grid(pos.x, pos.z);

            let prev_x = pos.x + imi.dx;
            let prev_y = pos.y + imi.dy;
            let prev_z = pos.z + imi.dz;

            let new_x = prev_x + velocity.x;
            let new_y = prev_y + velocity.y;
            let new_z = prev_z + velocity.z;

            let (new_col, new_row) = level.geom.blockmap.world_to_grid(new_x, new_z);

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
    world: &World,
    pending_moves: PendingMoves,
    level: &Level,
    blocklists: &mut [Vec<Entity>],
) {
    let mut query = world
        .query::<(Entity, &mut Position, &mut CurrentSector)>()
        .with::<&Active>();
    for (ent, pos, current_sector) in query.iter() {
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
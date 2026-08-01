use hecs::{Entity, World};
use wad_parser::{AABB, DoomMap};
use crate::{DB, FLOATSPEED, InstantMoveIntent, LinedefFlags, MAXRADIUS, MAXSPECHIT, MobjFlagCommand, MobjFlags, MobjInfo, MobjNum, MobjType, MonsterRotation, Position, Random, Target, WorldEvent, XSPEED, YSPEED, p_box_on_line_side};

#[derive(Debug, Clone)]
struct MoveContext {
    ceilingline_idx: Option<usize>, 
    ceiling_y: f32, 
    floor_y: f32, 
    dropoff_y: f32, 
    spec_hit: Vec<usize>,
}

pub fn p_move(
    ent: Entity,
	pos: &Position,
	rot: &MonsterRotation,
    mobj_type: &MobjType,
    mobj_info: &MobjInfo,
	imi: &mut InstantMoveIntent,
    map: &DoomMap,
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

    let try_x = pos.x + mobj_info.speed * XSPEED[move_dir as usize];
    let try_z = pos.z + mobj_info.speed * YSPEED[move_dir as usize];

    let (try_ok, float_ok) = p_try_move(
        ent,
        pos, 
        (try_x, pos.y, try_z), 
        mobj_type, 
        mobj_info, 
        imi, 
        map,
        world,
        random, 
        blocklists,
        world_events
    );
    
    if let Some(new_sector) = imi.new_sector {
        if !try_ok {
            if mobj_type.flags.contains(MobjFlags::FLOAT) && float_ok {
                if pos.y < map.sectors[new_sector].floorheight as f32 {
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
            imi.dy = map.sectors[new_sector].floorheight as f32 - pos.y;
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
	map: &DoomMap, 
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
    map: &DoomMap,
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
        min_y: goal_pos.2 - mobj_info.radius,
        max_y: goal_pos.2 + mobj_info.radius,
    };

	let (min_col, min_row) = map.blockmap.world_to_grid(bbox.min_x - MAXRADIUS, bbox.min_y - MAXRADIUS);
    let (max_col, max_row) = map.blockmap.world_to_grid(bbox.max_x + MAXRADIUS, bbox.max_y + MAXRADIUS);

    for r in min_row..=max_row {
        for c in min_col..=max_col {
            let idx = r * map.blockmap.col_num + c;

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

            for &line_idx in map.blockmap.blocklists[idx].iter() {
                if !pit_check_line(ctx, mobj_type, bbox, line_idx, map) {
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
    line_idx: usize,
    map: &DoomMap,
) -> bool {
    let line = map.linedefs[line_idx];

    let v1 = map.vertices[line.v1 as usize];
    let v2 = map.vertices[line.v2 as usize];

    let vmin_x = v1.x.min(v2.x) as f32;
    let vmax_x = v1.x.max(v2.x) as f32;
    let vmin_y = v1.y.min(v2.y) as f32;
    let vmax_y = v1.y.max(v2.y) as f32;
    
    if bbox.max_x < vmin_x
        || bbox.min_x > vmax_x
        || bbox.max_y < vmin_y
        || bbox.min_y > vmax_y
    {
        return true;
    }

    if p_box_on_line_side(&bbox, &line, map) != -1 {
        return true;
    }

    if !mobj_type.flags.contains(MobjFlags::MISSILE) {
        if (line.flags & LinedefFlags::BLOCKING.bits() as i16) != 0 {
            return false;
        }

        if mobj_type.type_ != MobjNum::Player && (line.flags & LinedefFlags::BLOCK_MONSTER.bits() as i16) != 0 {
            return false;
        }
    }

	if line.sidenum[0] == u16::MAX || line.sidenum[1] == u16::MAX { 
        return false; 
    }

	let front_sidedef = map.sidedefs[line.sidenum[0] as usize];
    let back_sidedef = map.sidedefs[line.sidenum[1] as usize];

    let front = map.sectors[front_sidedef.sector as usize];
    let back = map.sectors[back_sidedef.sector as usize];

    let open_top = front.ceilingheight.min(back.ceilingheight) as f32;
    let floor_high = front.floorheight.max(back.floorheight) as f32;
    let floor_low = front.floorheight.min(back.floorheight) as f32;

    if open_top < ctx.ceiling_y {
        ctx.ceiling_y = open_top;
        ctx.ceilingline_idx = Some(line_idx);
    }

    if floor_high > ctx.floor_y {
        ctx.floor_y = floor_high;
    }

    if floor_low < ctx.dropoff_y {
        ctx.dropoff_y = floor_low;
    }

    if line.special != 0 {
        if ctx.spec_hit.len() < 8 {
            ctx.spec_hit.push(line_idx);
        }
    }

    true
}

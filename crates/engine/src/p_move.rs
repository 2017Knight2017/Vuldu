use hecs::{Entity, World};
use wad_parser::{AABB, DoomMap};
use crate::{DB, FLOATSPEED, InstantMoveIntent, LinedefFlags, MAXRADIUS, MobjFlags, MobjInfo, MobjNum, MobjType, MonsterRotation, Position, Random, Target, WorldEvent, XSPEED, YSPEED};

pub fn p_move(
    ent: Entity,
	pos: &Position,
	rot: &MonsterRotation,
    mobj_type: &mut MobjType,
    mobj_info: &MobjInfo,
	imi: &mut InstantMoveIntent,
    map: &DoomMap,
    world: &World,
    random: &mut Random,
    blocklists: &[Vec<Entity>], 
    world_events: &mut Vec<WorldEvent>
) -> bool {
    let try_x = pos.x + mobj_info.speed * XSPEED[rot.move_dir as usize];
    let try_z = pos.z + mobj_info.speed * YSPEED[rot.move_dir as usize];

    let goal_sector_idx = map.get_sector_by_pos(try_x, try_z);

    let (try_ok, float_ok) = p_try_move(
        ent,
        pos, 
        (try_x, pos.y, try_z), 
        mobj_type, 
        mobj_info, 
        goal_sector_idx, 
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

                mobj_type.flags.insert(MobjFlags::IN_FLOAT);
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
            mobj_type.flags.remove(MobjFlags::IN_FLOAT);
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
	goal_sector_idx: usize,
    imi: &mut InstantMoveIntent,
	map: &DoomMap, 
    world: &World,
    random: &mut Random,
    blocklists: &[Vec<Entity>],
    world_events: &mut Vec<WorldEvent>
) -> (bool, bool) {
	// (try_ok, float_ok)

	if !p_check_pos(
        ent,
        mobj_type,
        goal_pos,
        mobj_info,
        //imi,
        map,
        world,
        random,
        blocklists,
        world_events
    ) {
		return (false, false)
	}

    let sector = map.sectors[goal_sector_idx];

	if !mobj_type.flags.contains(MobjFlags::NO_CLIP) {
		if ((sector.ceilingheight - sector.floorheight) as f32) < mobj_info.height {
            return (false, false);
        }

        if !mobj_type.flags.contains(MobjFlags::TELEPORT) &&
            sector.ceilingheight as f32 - pos.y < mobj_info.height
        {
            return (false, true);
        }

        if !mobj_type.flags.contains(MobjFlags::TELEPORT) &&
            sector.floorheight as f32 - pos.y > 24.0 
        {
            return (false, true);
        }

        //if !mobj_type.flags.intersects(MobjFlags::DROP_OFF | MobjFlags::FLOAT) &&
        //    sector.floorheight - dropoff > 24.0
        //{
        //    return (false, true);
        //}
	}

    imi.dx = goal_pos.0 - pos.x;
    imi.dz = goal_pos.2 - pos.z;
    imi.new_sector = Some(map.get_sector_by_pos(goal_pos.0, goal_pos.2));

    (true, true)
}

pub fn p_check_pos(
    ent: Entity,
    mobj_type: &MobjType,
    goal_pos: (f32, f32, f32),
    mobj_info: &MobjInfo,
    //imi: &mut InstantMoveIntent,
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

            for &other_entity in &blocklists[idx] {
                let mut query = world.query_one::<(&MobjType, &Position, Option<&Target>)>(other_entity);
                if let Ok((other_type, other_pos, raw_target)) = query.get() {
                    if !pit_check_thing(ent, mobj_type, goal_pos, raw_target, other_entity, other_type, other_pos, random, world_events) {
                        return false;
                    }
                }
            }

            for &line_idx in &map.blockmap.blocklists[idx] {
                if !pit_check_line(mobj_type, bbox, line_idx, map) {
                    return false;
                }
            }
        }
    }

    true
}

pub fn pit_check_thing(
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

pub fn pit_check_line(
    mobj_type: &MobjType,
    bbox: AABB,
    line_idx: usize,
    //imi: &mut InstantMoveIntent,
    map: &DoomMap,
) -> bool {
    let line = map.linedefs[line_idx];

    let v1 = map.vertices[line.v1 as usize];
    let v2 = map.vertices[line.v2 as usize];

    let vmin_x = v1.x.min(v2.x) as f32;
    let vmax_x = v1.x.max(v2.x) as f32;
    let vmin_y = v1.y.min(v2.y) as f32;
    let vmax_y = v1.y.max(v2.y) as f32;
    
    if bbox.max_x <= vmin_x
        || bbox.min_x >= vmax_x
        || bbox.max_y <= vmin_y
        || bbox.min_y >= vmax_y
    {
        return true;
    }

    //if p_box_on_line_side(&ctx.tmbbox, line) != -1 {
    //    return true;
    //}

    if !mobj_type.flags.contains(MobjFlags::MISSILE) {
        if (line.flags & LinedefFlags::BLOCKING.bits() as i16) == 1 {
            return false;
        }

        if mobj_type.type_ != MobjNum::Player && (line.flags & LinedefFlags::BLOCK_MONSTER.bits() as i16) == 1 {
            return false;
        }
    }

	//if line.sidenum[0] == u16::MAX { return false; }
	//let front_sidedef = map.sidedefs[line.sidenum[0] as usize];
	//let front_sector_idx = front_sidedef.sector;
	//let front = map.sectors[front_sector_idx as usize];
//
	//let back = if line.sidenum[1] != u16::MAX {
	//	map.sectors[map.sidedefs[line.sidenum[1] as usize].sector as usize]
	//} else {
    //    return false;
    //};

    //let open_top = front.ceilingheight.min(back.ceilingheight);
    //let open_bottom = front.floorheight.max(back.floorheight);
    //let lowfloor = front.floorheight.min(back.floorheight);

    //if open_top < ctx.tmceilingz {
    //    ctx.tmceilingz = open_top;
    //    ctx.ceilingline_idx = Some(line_idx);
    //}
//
    //if open_bottom > ctx.tmfloorz {
    //    ctx.tmfloorz = open_bottom;
    //}
//
    //if lowfloor < ctx.tmdropoffz {
    //    ctx.tmdropoffz = lowfloor;
    //}
//
    //if line.special != 0 {
    //    if ctx.spechit.len() < 8 {
    //        ctx.spechit.push(line_idx);
    //    }
    //}

    true
}

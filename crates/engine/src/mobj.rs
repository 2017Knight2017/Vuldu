use crate::*;
use hecs::{Entity, World, EntityBuilder};
use wad_parser::map::Level;

pub fn spawn_mobj(
	level: & Level,
	world: &mut World, 
	random: &mut Random,
	mobj_type_raw: Option<MobjNum>, 
	x_raw: i16, 
	z_raw: i16, 
	angle_raw: i16,
	blocklists: &mut [Vec<Entity>]
) -> Option<Entity> {
	let x = x_raw as f32;
	let z = z_raw as f32;

	let (row, col) = level.geom.blockmap.world_to_grid(x, z);
	
	let mobj_type = match mobj_type_raw {
		Some(mobj) => mobj,
		None => return None
	};

	let normalized_deg = ((angle_raw as i32 % 360) + 360) % 360;
	let angle = (normalized_deg / 45) as u32 * ANG45;

	let dir_index = (normalized_deg / 45) as u32;
	let move_dir = Direction::try_from(dir_index).expect("Error while parsing move_dir");

	let db = DB.get().unwrap();
	let mobj_info = db.mobjinfo.get(&mobj_type)
		.expect(&format!("[FATAL] mobj_info with {:?} was not found!", mobj_type));

	let spawn_state = match mobj_info.spawn_state {
		Some(state) => state,
		None => return None
	};

	let spawn_state_data = db.states.get(&spawn_state)
    	.expect("Spawn state not found in database!");
	let cached_rotations = spawn_state_data.cached_rotations;

	let m_len = mobj_info.flags.len();
	let mut top_offset_shift: i16 = 0;
	if m_len > 0 {
		top_offset_shift = match mobj_info.flags[m_len-1] {
			MobjFlagNum::VertOffsetM31 => -31,
			MobjFlagNum::VertOffsetM2 => -2,
			MobjFlagNum::VertOffsetM1 => -1,
			MobjFlagNum::VertOffset1 => 1,
			MobjFlagNum::VertOffset2 => 2,
			MobjFlagNum::VertOffset3 => 3,
			MobjFlagNum::VertOffset4 => 4,
			MobjFlagNum::VertOffset5 => 5,
			_ => 0
		};
	}

	let flags = mobj_info.flags
		.iter()
		.map(|&a| MobjFlags::from(a))
		.collect::<MobjFlags>();

	let sector_idx = level.get_sector_by_pos(x, z);
	let y = if flags.contains(MobjFlags::SPAWN_CEILING) {
		level.state.sectors[sector_idx.0].ceil_h - mobj_info.height
	} else {
		level.state.sectors[sector_idx.0].floor_h
	};

	let tics_left = if spawn_state_data.tics > 0 {
		1 + (random.p() as i32) % spawn_state_data.tics
	} else { 0 };

	let mut entity_builder = EntityBuilder::new();
    
    entity_builder
		.add(Position { x, y, z, prev_x: x, prev_y: y, prev_z: z })
		.add(CurrentSector(sector_idx))
		.add(Velocity::default())
		.add(MobjType { type_: mobj_type, flags })
		.add(Health(mobj_info.spawn_health));

	if mobj_type == MobjNum::Player {
		entity_builder
			.add(PlayerMarker)
			.add(PlayerRotation { angle, prev_angle: angle })
        	.add(PlayerCamera { view_z: EYEHEIGHT, view_height: EYEHEIGHT, delta_view_height: 0.0, bob: 0.0 })
			.add(PlayerInventory::default())
			.add(PlayerStats::default())
        	.add(WeaponOverlay { state_idx: 0, tics: 0, sx: 0.0, sy: 0.0 });
	} else {
		entity_builder
			.add(Idle)
			.add(InstantMoveIntent { dx: 0.0, dy: 0.0, dz: 0.0, new_sector: None })
			.add(SpriteAnimation {
				cached_rotations, 
				top_offset_shift
			})
			.add(MonsterRotation { 
				move_dir: Some(move_dir),
				move_count: (random.p() & 0b111) as i32
			})
			.add(MobjAi {
				current_state: spawn_state,
    			tics_left,
    			threshold: 0,
				reaction_time: mobj_info.reaction_time,
			});
	};

	let entity = world.spawn(entity_builder.build());
	blocklists[row + level.geom.blockmap.col_num + col].push(entity);

    Some(entity)
}

pub fn spawn_all_things(
	world: &mut World, 
	level: &Level, 
	random: &mut Random, 
	player_entity: &mut Entity, 
	blocklists: &mut [Vec<Entity>]
) {
	let mut player_spawned = false;
	for thing in level.things.iter() {
		if thing.type_ == 1 {
            if player_spawned {
                continue;
            }
            player_spawned = true;
		}

		if let Some(thing_type) = MOBJTYPE_BY_DOOMEDNUM.get(&thing.type_) {
			let ent_opt = spawn_mobj(level, world, random, *thing_type, 
				thing.x, thing.y, thing.angle, blocklists);
			if thing.type_ == 1 { 
				*player_entity = ent_opt.unwrap(); 
			}
		}
	}
}

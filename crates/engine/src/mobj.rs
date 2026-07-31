use crate::*;
use hecs::World;
use wad_parser::map::DoomMap;

pub fn spawn_mobj(
	world: &mut World, 
	random: &mut Random,
	mobj_type_raw: Option<MobjNum>, 
	sector_idx: usize,
	x_raw: i16, 
	y_raw: i16, 
	z_raw: i16, 
	angle_raw: i16
) {
	let x = x_raw as f32;
	let y = y_raw as f32;
	let z = z_raw as f32;
	
	let mobj_type = match mobj_type_raw {
		Some(mobj) => mobj,
		None => return
	};

	let angle = (angle_raw + 270) as u32 % 360 / 45 * ANG45;
		
	let move_dir = Direction::try_from(angle_raw as u32 / 45).expect("Error while parsing move_dir");

	let db = DB.get().unwrap();
	let mobj_info = db.mobjinfo.get(&mobj_type)
		.expect(&format!("[FATAL] mobj_info with {:?} was not found!", mobj_type));

	let spawn_state = match mobj_info.spawn_state {
		Some(state) => state,
		None => return
	};

	let spawn_state_data = db.states.get(&spawn_state)
    	.expect("Spawn state not found in database!");
	let cached_rotations = spawn_state_data.cached_rotations;

	let m_len = mobj_info.flags.len();
	let mut top_offset_shift: i16 = 0;
	if m_len > 0 {
		top_offset_shift = match mobj_info.flags[m_len-1] {
			MobjFlagNum::VertOffsetM1 => -1,
			MobjFlagNum::VertOffsetM2 => -2,
			MobjFlagNum::VertOffset1 => 1,
			MobjFlagNum::VertOffset2 => 2,
			MobjFlagNum::VertOffset3 => 3,
			MobjFlagNum::VertOffset4 => 4,
			MobjFlagNum::VertOffset5 => 5,
			_ => 0
		}
	}

	let tics_left = if spawn_state_data.tics > 0 {
		1 + (random.p() as i32) % spawn_state_data.tics
	} else { 0 };

	let mut entity_builder = hecs::EntityBuilder::new();
    
    entity_builder
		.add(Position { x, y, z, prev_x: x, prev_y: y, prev_z: z })
		.add(CurrentSector(sector_idx))
		.add(Velocity::default())
		.add(MobjType { 
			type_: mobj_type, 
			flags: mobj_info.flags
				.iter()
				.map(|&a| MobjFlags::from(a))
				.collect() 
			})
		.add(Health(mobj_info.spawn_health));

	if mobj_type == MobjNum::Player {
		entity_builder
			.add(PlayerMarker)
			.add(Active)
			.add(PlayerRotation { angle, prev_angle: angle })
        	.add(PlayerCamera { view_z: 41.0, view_height: 41.0, delta_view_height: 0.0, bob: 0.0 })
        	.add(PlayerStats::default())
        	.add(PlayerInventory { 
        	    ready_weapon: 1, 
        	    pending_weapon: 1, 
        	    backpack: false, 
        	    cards: [false; NUMCARDS], 
        	    weapon_owned: [false; NUMWEAPONS], 
        	    ammo: [50, 0, 0, 0], 
        	    max_ammo: [200, 50, 50, 300] 
        	})
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
				move_count: (random.p() & 0xF) as i32
			})
			.add(MobjAi {
				current_state: spawn_state,
    			tics_left,
    			action: spawn_state_data.action,
    			threshold: 0,
				reaction_time: mobj_info.reaction_time,
			});
	};

	for flag in &mobj_info.flags {
		match flag {
			MobjFlagNum::Solid => entity_builder.add(Solid),
			MobjFlagNum::CountKill => entity_builder.add(CountKill),
			MobjFlagNum::CountItem => entity_builder.add(CountItem),
			MobjFlagNum::Special => entity_builder.add(Special),
			MobjFlagNum::Pickup => entity_builder.add(Pickup),
			_ => { continue; }
		};
	}

    world.spawn(entity_builder.build());
}

pub fn spawn_all_things(world: &mut World, map: &DoomMap, random: &mut Random) {
	let mut player_spawned = false;
	for thing in map.things.iter() {
		if thing.type_ == 1 {
            if player_spawned {
                continue;
            }
            player_spawned = true;
		}

		let sector_idx = map.get_sector_by_pos(thing.x as f32, thing.y as f32);
		let sector = map.sectors[sector_idx];

		if let Some(thing_type) = MOBJTYPE_BY_DOOMEDNUM.get(&thing.type_) {
			spawn_mobj(world, random, *thing_type, sector_idx, -thing.x, sector.floorheight, thing.y, thing.angle);
		}
	}
}

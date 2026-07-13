use crate::{
	constants::{NUMCARDS, NUMWEAPONS, MOBJTYPE_BY_DOOMEDNUM},
	data_tables::DB,
	components::*,
	enums::*,
	random::Random,
	player::*,
};
use hecs::World;
use wad_parser::map::DoomMap;

pub fn spawn_mobj(
	world: &mut World, 
	random: &mut Random,
	mobj_type_raw: Option<MobjType>, 
	sector_idx: usize,
	x_raw: i16, 
	y_raw: i16, 
	z_raw: i16, 
	angle_raw: i16
) {
	let x = x_raw as f32;
	let y = y_raw as f32;
	let z = z_raw as f32;
	
    let angle = if angle_raw < 0 {
		(360 + angle_raw - 90) as u32 / 45 * 0x20000000
	} else {
		angle_raw as u32 / 45 * 0x20000000
	};

	let mobj_type = match mobj_type_raw {
		Some(mobj) => mobj,
		None => return
	};

	let db = DB.get().expect("DB has not been initialized!");
	let mobj_info = db.mobjinfo.get(&mobj_type).expect(&format!("[FATAL] mobj_info with {:?} was not found!", mobj_type));

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
			MobjFlag::VertOffsetM1 => -1,
			MobjFlag::VertOffsetM2 => -2,
			MobjFlag::VertOffset1 => 1,
			MobjFlag::VertOffset2 => 2,
			MobjFlag::VertOffset3 => 3,
			MobjFlag::VertOffset4 => 4,
			MobjFlag::VertOffset5 => 5,
			_ => 0
		}
	}

	let tics_left = if spawn_state_data.tics > 0 {
		1 + (random.p() as i32) % spawn_state_data.tics
	} else { 0 };

	let mut entity_builder = hecs::EntityBuilder::new();
    
    entity_builder
		.add(Transform { x, y, z, prev_x: x, prev_y: y, prev_z: z, angle, prev_angle: angle })
		.add(CurrentSector(sector_idx))
		.add(Velocity::default())
		.add(BoundingBox { radius: mobj_info.radius, height: mobj_info.height })
		.add(SpriteAnimation {
			current_state: mobj_info.spawn_state, tics_left, cached_rotations, top_offset_shift});

	for flag in &mobj_info.flags {
		match flag {
			MobjFlag::Solid => entity_builder.add(Solid),
			MobjFlag::CountKill => entity_builder.add(CountKill),
			MobjFlag::Shootable => {
				entity_builder.add(Health { current: mobj_info.spawn_health, max: mobj_info.spawn_health })
					.add(PainReaction { chance: mobj_info.pain_chance, sound: mobj_info.pain_sound })
					.add(MonsterBrainConfig {
                	    spawn_state: mobj_info.spawn_state,
                	    see_state: mobj_info.see_state,
                	    death_state: mobj_info.death_state,
                	    death_sound: mobj_info.death_sound,
                	    missile_state: mobj_info.missile_state,
                	    pain_state: mobj_info.pain_state,
                	    xdeath_state: mobj_info.xdeath_state,
                	    raise_state: mobj_info.raise_state
                	})
					.add(Shootable)
			},
			MobjFlag::CountItem => entity_builder.add(CountItem),
			MobjFlag::Special => entity_builder.add(Special),
			MobjFlag::Pickup => entity_builder.add(Pickup),
			_ => { continue; }
		};
	}

    if mobj_info.speed > 0.0 {
        entity_builder.add(Physics {
            speed: mobj_info.speed,
            mass: mobj_info.mass,
        });
    }

	if mobj_type == MobjType::Player {
		entity_builder.add(PlayerMarker)
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
	};

    world.spawn(entity_builder.build());
}

pub fn spawn_all_things(world: &mut World, map: &DoomMap, random: &mut Random) {
	for thing in map.things.iter() {
		let sector_idx = map.get_sector_by_pos(thing.x as f32, thing.y as f32);
		let sector = map.sectors[sector_idx];

		if let Some(thing_type) =  MOBJTYPE_BY_DOOMEDNUM.get(&thing.type_) {
			spawn_mobj(world, random, *thing_type, sector_idx, -thing.x, sector.floorheight, thing.y, thing.angle);
		}
	};
}

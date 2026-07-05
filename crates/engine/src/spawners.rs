use crate::{
	constants::{NUMCARDS, NUMWEAPONS, MOBJTYPE_BY_DOOMEDNUM},
	data_tables::DB,
	components::*,
	enums::*
};
use hecs::World;
use wad_parser::map::DoomMap;

pub fn spawn_mobj(
	world: &mut World, 
	mobj_type_raw: Option<MobjType>, 
	x_raw: i16, 
	y_raw: i16, 
	z_raw: i16, 
	angle_raw: i16
) {
	let x = x_raw as f32;
	let y = y_raw as f32;
	let z = z_raw as f32;

    let angle = angle_raw as u32 / 45 * 0x20000000;

	let mobj_type = match mobj_type_raw {
		Some(mobj) => mobj,
		None => return
	};

	let mobj_info = DB.get().expect("DB has not been initialized!").mobjinfo.get(&mobj_type).unwrap();

	let mut entity_builder = hecs::EntityBuilder::new();
    
    entity_builder
		.add(Transform { x, y, z, prev_x: x, prev_y: y, prev_z: z, angle, prev_angle: angle })
		.add(Velocity::default())
		.add(BoundingBox { radius: mobj_info.radius, height: mobj_info.height })
		.add(SpriteAnimation {current_state: mobj_info.spawn_state, tics_left: 8});

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
			_ => {continue;}
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

pub fn spawn_all_things(world: &mut World, map: &DoomMap) {
	for thing in map.things.iter() {
		let sector_idx = map.get_sector_by_pos(thing.x as f32, thing.y as f32);
		let sector = map.sectors[sector_idx];

		if let Some(thing_type) =  MOBJTYPE_BY_DOOMEDNUM.get(&thing.type_) {
			spawn_mobj(world, *thing_type, -thing.x, sector.floorheight, thing.y, thing.angle);
		}
	};
}

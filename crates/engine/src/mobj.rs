use crate::*;
use hecs::{CommandBuffer, Entity, EntityBuilder, World};
use wad_parser::{
	map::Level,
	wad_types::{MapThing, ThingFlags},
};

pub fn spawn_mobj(
	level: &Level,
	world: &mut World,
	random: &mut Random,
	thing: &MapThing,
	blocklists: &mut [Vec<Entity>],
	cfg: &GameConfig,
) -> Option<Entity> {
	let thing_type = (*MOBJTYPE_BY_DOOMEDNUM.get(&thing.type_)?)?;
	let thing_flags = ThingFlags::from_bits(thing.flags).unwrap_or(ThingFlags::NONE);

	let easy_cond = !thing_flags.contains(ThingFlags::SPAWN_ON_EASY)
		&& (cfg.skill == SkillLevel::Baby || cfg.skill == SkillLevel::Easy);
	let medium_cond =
		!thing_flags.contains(ThingFlags::SPAWN_ON_MED) && cfg.skill == SkillLevel::Medium;
	let hard_cond = !thing_flags.contains(ThingFlags::SPAWN_ON_HARD)
		&& (cfg.skill == SkillLevel::Hard || cfg.skill == SkillLevel::Nightmare);

	let singleplr_cond =
		thing_flags.contains(ThingFlags::NOT_SINGLEPLR) && !(cfg.dmatch || cfg.coop);
	let coop_cond = thing_flags.contains(ThingFlags::NOT_COOP) && cfg.coop;
	let dmatch_cond = thing_flags.contains(ThingFlags::NOT_DMATCH) && cfg.dmatch;

	if easy_cond || medium_cond || hard_cond || singleplr_cond || coop_cond || dmatch_cond {
		return None;
	}

	let x = thing.x as f32;
	let z = thing.y as f32;

	let (col, row) = level.geom.blockmap.world_to_grid(x, z);

	let normalized_deg = ((thing.angle as i32 % 360) + 360) % 360;
	let mut angle = (normalized_deg / 45) as u32 * ANG45;
	if thing_type == MobjNum::Player {
		angle = angle.wrapping_sub(ANG90);
	}

	let dir_index = (normalized_deg / 45) as u32;
	let move_dir = Direction::try_from(dir_index).expect("Error while parsing move_dir");

	let db = DB.get().unwrap();
	let mobj_info = db
		.mobjinfo
		.get(&thing_type)
		.unwrap_or_else(|| panic!("[FATAL] mobj_info with {:?} was not found!", thing_type));

	let mut mobj_flags = mobj_info
		.flags
		.iter()
		.map(|&a| MobjFlags::from(a))
		.collect::<MobjFlags>();

	if thing_type != MobjNum::Player && mobj_flags.contains(MobjFlags::SHOOTABLE) && cfg.no_monsters
	{
		return None;
	}

	if thing_flags.contains(ThingFlags::AMBUSH) {
		mobj_flags.insert(MobjFlags::AMBUSH);
	}

	let spawn_state = mobj_info.spawn_state?;

	let spawn_state_data = db
		.states
		.get(&spawn_state)
		.expect("Spawn state not found in database!");
	let cached_rotations = spawn_state_data.cached_rotations;
	let full_bright = spawn_state_data.frame & (1 << 15) != 0;

	let m_len = mobj_info.flags.len();
	let mut top_offset_shift: i16 = 0;
	if m_len > 0 {
		top_offset_shift = match mobj_info.flags[m_len - 1] {
			MobjFlagNum::VertOffsetM31 => -31,
			MobjFlagNum::VertOffsetM2 => -2,
			MobjFlagNum::VertOffsetM1 => -1,
			MobjFlagNum::VertOffset1 => 1,
			MobjFlagNum::VertOffset2 => 2,
			MobjFlagNum::VertOffset3 => 3,
			MobjFlagNum::VertOffset4 => 4,
			MobjFlagNum::VertOffset5 => 5,
			_ => 0,
		};
	}

	let sector_idx = level.get_sector_by_pos(x, z);
	let y = if mobj_flags.contains(MobjFlags::SPAWN_CEILING) {
		level.state.sectors[sector_idx.0].ceil_h - mobj_info.height
	} else {
		level.state.sectors[sector_idx.0].floor_h
	};

	let tics_left = if spawn_state_data.tics > 0 {
		1 + (random.p() as i32) % spawn_state_data.tics
	} else {
		0
	};

	let mut entity_builder = EntityBuilder::new();

	entity_builder
		.add(Position {
			x,
			y,
			z,
			prev_x: x,
			prev_y: y,
			prev_z: z,
		})
		.add(CurrentSector(sector_idx))
		.add(Velocity::default())
		.add(MobjType {
			type_: thing_type,
			flags: mobj_flags,
		})
		.add(Health(mobj_info.spawn_health));

	if thing_type == MobjNum::Player {
		entity_builder
			.add(PlayerMarker)
			.add(PlayerRotation {
				angle,
				prev_angle: angle,
			})
			.add(PlayerCamera {
				view_z: EYEHEIGHT,
				view_height: EYEHEIGHT,
				delta_view_height: 0.0,
				bob: 0.0,
			})
			.add(PlayerInventory::default())
			.add(PlayerStats::default())
			.add(WeaponOverlay {
				state_idx: 0,
				tics: 0,
				sx: 0.0,
				sy: 0.0,
			});
	} else {
		entity_builder
			.add(InstantMoveIntent {
				dx: 0.0,
				dy: 0.0,
				dz: 0.0,
				new_sector: None,
			})
			.add(SpriteAnimation {
				cached_rotations,
				top_offset_shift,
				full_bright,
			})
			.add(MonsterRotation {
				move_dir: Some(move_dir),
				move_count: (random.p() & 0b111) as i32,
			})
			.add(MobjAi {
				current_state: spawn_state,
				tics_left,
				threshold: 0,
				reaction_time: mobj_info.reaction_time,
			});
	};

	let entity = world.spawn(entity_builder.build());
	blocklists[row * level.geom.blockmap.col_num + col].push(entity);

	Some(entity)
}

pub fn spawn_all_things(
	world: &mut World,
	level: &Level,
	random: &mut Random,
	player_entity: &mut Entity,
	blocklists: &mut [Vec<Entity>],
	cfg: &GameConfig,
) {
	let mut player_spawned = false;
	for thing in level.things.iter() {
		if thing.type_ == 1 {
			if player_spawned {
				continue;
			}
			player_spawned = true;
		}

		let ent_opt = spawn_mobj(level, world, random, thing, blocklists, cfg);
		if thing.type_ == 1 {
			*player_entity = ent_opt.unwrap();
		}
	}
}

pub fn kill_mobj(
	ent: Entity,
	world: &World,
	level: &Level,
	cmd: &mut CommandBuffer,
	blocklists: &mut [Vec<Entity>],
) {
	let Ok(pos) = world.get::<&Position>(ent) else {
		return;
	};

	let (col, row) = level.geom.blockmap.world_to_grid(pos.x, pos.z);
	blocklists[row * level.geom.blockmap.col_num + col].retain(|&e| e != ent);

	cmd.despawn(ent);
}

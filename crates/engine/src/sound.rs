use hecs::{CommandBuffer, Entity, QueryBorrow};
use wad_parser::{MapLinedef, MapSidedef, to_u64};
use crate::{Active, CurrentSector, Database, DynSector, Idle, LinedefFlags, MobjAi, MobjNum, MobjType, PlayerShoot, Position, Random, SfxEvent, SpriteAnimation, Target};

pub fn propagate_sound_system(
	mut query: QueryBorrow<'_, (Entity, &CurrentSector, &PlayerShoot)>, 
	command_buffer: &mut CommandBuffer,
	sectors: &mut [DynSector],
	linedefs: &[MapLinedef],
	sidedefs: &[MapSidedef]
) {
	for (entity, current_sector, _player_shoot) in query.iter() {
		propagate_sound_internal(current_sector.0, 0, sectors, linedefs, sidedefs, entity);

		command_buffer.remove_one::<PlayerShoot>(entity);
	}
}

fn propagate_sound_internal(
	current_sector_idx: usize,
	times_blocked: u32,
	sectors: &mut [DynSector],
	linedefs: &[MapLinedef],
	sidedefs: &[MapSidedef],
	sound_target: Entity
) {
	if times_blocked >= 2 || times_blocked >= sectors[current_sector_idx].sound_traversed { 
		return; 
	}

	if times_blocked < sectors[current_sector_idx].sound_traversed {
        sectors[current_sector_idx].sound_traversed = times_blocked;
		sectors[current_sector_idx].sound_target = Some(sound_target);
    } else {
        return;
    }

	let lines = sectors[current_sector_idx].lines.clone();
	lines.iter().for_each(|&i| {
		if linedefs[i].flags & LinedefFlags::TWO_SIDED.bits() as i16 == 0 {
			return;
		} 
			
		let front_sector_idx = sidedefs[linedefs[i].sidenum[0] as usize].sector;
		let back_sector_idx = sidedefs[linedefs[i].sidenum[1] as usize].sector;
		let other_sector_idx = if current_sector_idx as i16 == front_sector_idx {
			back_sector_idx as usize
		} else {
			front_sector_idx as usize
		};

		let current_sector_props = sectors[current_sector_idx].props;
		let other_sector_props = sectors[other_sector_idx as usize].props;
		let is_shut = current_sector_props.ceilingheight - current_sector_props.floorheight == 0
			|| other_sector_props.ceilingheight - other_sector_props.floorheight == 0;

		if is_shut { return; }

		if linedefs[i].flags & LinedefFlags::SOUND_BLOCK.bits() as i16 == 1 {
			propagate_sound_internal(other_sector_idx, times_blocked + 1, sectors, linedefs, sidedefs, sound_target);
		} else {
			propagate_sound_internal(other_sector_idx, times_blocked, sectors, linedefs, sidedefs, sound_target);
		}
	});
}

pub fn wake_up_monster(
	entity: Entity,
	pos: &Position,
	mobj_type: &MobjType,
	sound_target: Entity,
	sprite_anim: &mut SpriteAnimation,
	ai: &mut MobjAi,
	random: &mut Random,
	command_buffer: &mut CommandBuffer, 
	audio_buffer: &mut Vec<SfxEvent>, 
	db: &Database
) {
	let mobj_info = db.mobjinfo.get(&mobj_type.type_).unwrap();

	if let (Some(see_state_num), Some(mut see_sound)) = (mobj_info.see_state, mobj_info.see_sound) {
		let see_state = db.states.get(&see_state_num).unwrap();
		
		ai.current_state = see_state_num;
		ai.action = see_state.action;
		ai.tics_left = see_state.tics + (random.p() & 0b111) as i32;
		sprite_anim.cached_rotations = see_state.cached_rotations.clone();
		
		if see_sound.starts_with(b"DSPOSIT") {
			see_sound[7] = random.p() % 3 + b'1';
		} else if see_sound.starts_with(b"DSBGSIT") {
			see_sound[7] = random.p() % 2 + b'1';
		}

		let pos = if mobj_type.type_ == MobjNum::Spider || mobj_type.type_ == MobjNum::Cyborg {
			None
		} else {
			Some((pos.x, pos.y, pos.z))
		};
			
		audio_buffer.push(SfxEvent { sfx_id: to_u64(&see_sound), pos })
	}

	command_buffer.remove_one::<Idle>(entity);
    command_buffer.insert(entity, (Target(sound_target), Active));
}

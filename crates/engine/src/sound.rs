use hecs::{CommandBuffer, Entity, QueryBorrow};
use wad_parser::{MapLinedef, MapSidedef, Sector};
use crate::{Active, CurrentSector, DB, LinedefFlags, MobjType, PlayerShoot, Sleeping, SpriteAnimation};

pub fn propagate_sound_system(
	mut query: QueryBorrow<'_, (Entity, &CurrentSector, &PlayerShoot)>, 
	command_buffer: &mut CommandBuffer,
	sectors: &mut Vec<Sector>,
	linedefs: &Vec<MapLinedef>,
	sidedefs: &Vec<MapSidedef>
) {
	for (entity, current_sector, _player_shoot) in query.iter() {
		propagate_sound_internal(current_sector.0, 0, sectors, linedefs, sidedefs);

		command_buffer.remove_one::<PlayerShoot>(entity);
	}
}

fn propagate_sound_internal(
	current_sector_idx: usize,
	times_blocked: u32,
	sectors: &mut Vec<Sector>,
	linedefs: &[MapLinedef],
	sidedefs: &[MapSidedef]
) {
	if times_blocked >= 2 || times_blocked >= sectors[current_sector_idx].sound_traversed { 
		return; 
	}

	if times_blocked < sectors[current_sector_idx].sound_traversed {
        sectors[current_sector_idx].sound_traversed = times_blocked;
    } else {
        return;
    }

	let lines = sectors[current_sector_idx].lines.clone();
	lines.iter().for_each(|&i| {
		let twosided_flag = 1 << LinedefFlags::TwoSided as u32;
		if linedefs[i].flags & twosided_flag == 0 {
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

		let soundblock_flag = 1 << LinedefFlags::SoundBlock as u32;
		if linedefs[i].flags & soundblock_flag != 0 {
			propagate_sound_internal(other_sector_idx, times_blocked + 1, sectors, linedefs, sidedefs);
		} else {
			propagate_sound_internal(other_sector_idx, times_blocked, sectors, linedefs, sidedefs);
		}
	});
}

pub fn check_sound_system(
	mut query: QueryBorrow<'_, (Entity, &CurrentSector, &SpriteAnimation, &MobjType, &Sleeping)>, 
	command_buffer: &mut CommandBuffer, 
	sectors: &Vec<Sector>
) {
	for (entity, current_sector, sprite_anim, mobj_type, _sleeping) in query.iter() {
		if sectors[current_sector.0].sound_traversed == u32::MAX { continue; }

		let db = DB.get().expect("DB has not been initialized!");

		if let Some(see_state_num) = db.mobjinfo.get(&mobj_type.0).unwrap().see_state {
			let see_state = db.states.get(&see_state_num).unwrap();
			let new_anim = SpriteAnimation {
                current_state: Some(see_state_num),
                tics_left: see_state.tics,
                cached_rotations: see_state.cached_rotations.clone(),
                top_offset_shift: sprite_anim.top_offset_shift
            };

            command_buffer.remove::<(Sleeping, SpriteAnimation)>(entity);
            command_buffer.insert(entity, (Active, new_anim));
		} else {
            command_buffer.remove_one::<Sleeping>(entity);
            command_buffer.insert_one(entity, Active);
        }
	}
}

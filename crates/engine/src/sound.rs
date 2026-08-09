use hecs::{CommandBuffer, Entity, QueryBorrow, With};
use wad_parser::{Level, LineFlags, LineId, SectorId, to_u64};
use crate::{Active, CurrentSector, Database, Idle, MAXSOUNDBLOCKS, MobjAi, MobjNum, MobjType, PlayerShoot, Position, Random, SfxEvent, SpriteAnimation, Target};

pub struct Traversal {
    generation: u32,
    lines: Vec<u32>, 
    sectors: Vec<u32>, 
    sector_cost: Vec<u8>, 
}

impl Traversal {
    pub fn for_level(level: &Level) -> Self {
        Self {
            generation:  0,
            lines: vec![0; level.geom.lines.len()],
            sectors: vec![0; level.state.sectors.len()],
            sector_cost: vec![0; level.state.sectors.len()],
        }
    }

    pub fn begin(&mut self) -> Pass<'_> {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 { 
            self.lines.fill(0);
            self.sectors.fill(0);
            self.generation = 1;
        }
        Pass { traversal: self }
    }
}
pub struct Pass<'a> { traversal: &'a mut Traversal }

impl Pass<'_> {
    pub fn visit_line(&mut self, l: LineId) -> bool {
        let slot = &mut self.traversal.lines[l.0 as usize];

        if *slot == self.traversal.generation { 
			false 
		} else { 
			*slot = self.traversal.generation; 
			true 
		}
    }

    pub fn improve_sector(&mut self, sector_id: SectorId, cost: u8) -> bool {
        let i = sector_id.0 as usize;

        if self.traversal.sectors[i] != self.traversal.generation {
            self.traversal.sectors[i] = self.traversal.generation;
            self.traversal.sector_cost[i] = cost;
            true
        } else if cost < self.traversal.sector_cost[i] {
            self.traversal.sector_cost[i] = cost;
            true
        } else {
            false
        }
    }
}

pub fn propagate_sound(
    level: &Level,
    traversal: &mut Traversal,
    sound_targets: &mut [Option<Entity>],
    from: SectorId,
    emitter: Entity,
) {
    let mut pass = traversal.begin();
    let mut stack = vec![(from, 0u8)];

    while let Some((sector_id, blocks)) = stack.pop() {
        if blocks > MAXSOUNDBLOCKS { continue; }
        if !pass.improve_sector(sector_id, blocks) { continue; }

        sound_targets[sector_id.0] = Some(emitter);

        for &line_id in level.geom.sector_lines[sector_id.0].iter() {
            let open = match level.get_opening(line_id) {
				Some(opening) => opening,
				None => continue
			};

            if open.top <= open.floor_high { continue; }

            let line = &level.geom.lines[line_id.0];
			if let Some(other_sector) = level.get_other_sector(line_id, sector_id) {
				let next_blocks = blocks + line.flags.contains(LineFlags::SOUND_BLOCK) as u8;
				stack.push((other_sector, next_blocks));
			}
        }
    }
}

pub fn propagate_sound_system(
	mut query: QueryBorrow<'_, With<(Entity, &CurrentSector), &PlayerShoot>>, 
	level: &Level,
	sound_targets: &mut [Option<Entity>],
	traversal: &mut Traversal,
	command_buffer: &mut CommandBuffer
) {
	for (ent, current_sector) in query.iter() {
		propagate_sound(level, traversal, sound_targets, current_sector.0, ent);

		command_buffer.remove_one::<PlayerShoot>(ent);
	}
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

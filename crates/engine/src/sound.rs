use crate::{CurrentSector, MAXSOUNDBLOCKS, PlayerShoot};
use hecs::{CommandBuffer, Entity, World};
use wad_parser::{Level, LineFlags, LineId, SectorId};

pub struct Traversal {
	generation: u32,
	lines: Vec<u32>,
	sectors: Vec<u32>,
	sector_cost: Vec<u8>,
}

impl Traversal {
	pub fn for_level(level: &Level) -> Self {
		Self {
			generation: 0,
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
		Pass(self)
	}
}
pub struct Pass<'a>(pub &'a mut Traversal);

impl Pass<'_> {
	pub fn visit_line(&mut self, l: LineId) -> bool {
		let slot = &mut self.0.lines[l.0];

		if *slot == self.0.generation {
			false
		} else {
			*slot = self.0.generation;
			true
		}
	}

	pub fn improve_sector(&mut self, sector_id: SectorId, cost: u8) -> bool {
		let i = sector_id.0;

		if self.0.sectors[i] != self.0.generation {
			self.0.sectors[i] = self.0.generation;
			self.0.sector_cost[i] = cost;
			true
		} else if cost < self.0.sector_cost[i] {
			self.0.sector_cost[i] = cost;
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
		if blocks > MAXSOUNDBLOCKS {
			continue;
		}
		if !pass.improve_sector(sector_id, blocks) {
			continue;
		}

		sound_targets[sector_id.0] = Some(emitter);

		for &line_id in level.geom.sector_lines[sector_id.0].iter() {
			let Some(open) = level.get_opening(line_id) else {
				continue;
			};

			if open.top <= open.floor_high {
				continue;
			}

			let line = &level.geom.lines[line_id.0];
			if let Some(other_sector) = level.get_other_sector(line_id, sector_id) {
				let next_blocks = blocks + line.flags.contains(LineFlags::SOUND_BLOCK) as u8;
				stack.push((other_sector, next_blocks));
			}
		}
	}
}

pub fn propagate_sound_system(
	world: &World,
	level: &Level,
	sound_targets: &mut [Option<Entity>],
	traversal: &mut Traversal,
	command_buffer: &mut CommandBuffer,
) {
	let mut query = world
		.query::<(Entity, &CurrentSector)>()
		.with::<&PlayerShoot>();
	for (ent, current_sector) in query.iter() {
		propagate_sound(level, traversal, sound_targets, current_sector.0, ent);

		command_buffer.remove_one::<PlayerShoot>(ent);
	}
}

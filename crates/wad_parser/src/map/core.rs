use crate::{AABB, ActiveEffect, WadManager, wad_types::*};
use fixedbitset::FixedBitSet;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::mem::size_of;
use std::ptr::read_unaligned;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SectorId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SideId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubsectorId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextureId(pub u32);

#[derive(Debug, Clone, Default)]
pub struct Blockmap {
	pub origin_x: i16,
	pub origin_z: i16,
	pub col_num: usize,
	pub row_num: usize,
	pub blocklists: Vec<Vec<LineId>>,
}

#[derive(Debug, Clone, Default)]
pub struct RejectTable(pub Option<Vec<u8>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlopeType {
	Horizontal,
	Vertical,
	Positive,
	Negative,
}

fn get_slope_type(dx: f32, dy: f32) -> SlopeType {
	if dx == 0.0 {
		SlopeType::Vertical
	} else if dy == 0.0 {
		SlopeType::Horizontal
	} else if (dx > 0.0 && dy > 0.0) || (dx < 0.0 && dy < 0.0) {
		SlopeType::Positive
	} else {
		SlopeType::Negative
	}
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct LineFlags: u16 {
		const NONE = 0;
		const BLOCKING = 1 << 0;
		const BLOCK_MONSTER = 1 << 1;
		const TWO_SIDED = 1 << 2;
		const DONT_PEG_TOP = 1 << 3;
		const DONT_PEG_BOTTOM = 1 << 4;
		const SECRET = 1 << 5;
		const SOUND_BLOCK = 1 << 6;
		const DONT_DRAW = 1 << 7;
		const MAPPED = 1 << 8;
	}
}

#[derive(Debug)]
pub struct Line {
	pub v1: VertexId,
	pub v2: VertexId,
	pub flags: LineFlags,
	pub special: u16,
	pub tag: u16,
	pub sides: (Option<SideId>, Option<SideId>),
	pub delta: (f32, f32),
	pub bbox: AABB,
	pub slope: SlopeType,
}

#[derive(Debug, Default)]
pub struct Side {
	pub sector: SectorId,
}

#[derive(Debug, Default)]
pub struct Geometry {
	pub vertices: Vec<(f32, f32)>,
	pub lines: Vec<Line>,
	pub sides: Vec<Side>,
	pub subsectors: Vec<MapSubsector>,
	pub segs: Vec<MapSegment>,
	pub nodes: Vec<MapNode>,
	pub reject_table: RejectTable,
	pub blockmap: Blockmap,
	pub sector_lines: Vec<Vec<LineId>>,
	pub tag_sectors: FxHashMap<u16, Vec<SectorId>>,
	pub movable_sectors: FixedBitSet,
	pub subsector_sector: Vec<SectorId>,
}

#[derive(Debug, Default)]
pub struct SectorState {
	pub floor_h: f32,
	pub ceil_h: f32,
	pub light: u16,
	pub floor_tex: TextureId,
	pub ceil_tex: TextureId,
	pub(crate) floorpic: [u8; 8],
	pub(crate) ceilingpic: [u8; 8],
	pub special: u16,
	pub tag: u16,
	pub active_mover: Option<EffectId>,
}

#[derive(Debug, Default)]
pub struct SideState {
	pub col_offset: i16,
	pub row_offset: i16,
	pub top_tex: Option<TextureId>,
	pub bottom_tex: Option<TextureId>,
	pub mid_tex: Option<TextureId>,
	pub(crate) toptexture: [u8; 8],
	pub(crate) bottomtexture: [u8; 8],
	pub(crate) midtexture: [u8; 8],
}

#[derive(Debug, Default)]
pub struct LineState {
	pub used: bool,
	pub mapped: bool,
}

#[derive(Debug, Default)]
pub struct LevelState {
	pub sectors: Vec<SectorState>,
	pub sides: Vec<SideState>,
	pub lines: Vec<LineState>,
	pub effects: Vec<ActiveEffect>,
}

#[derive(Debug, Default)]
pub struct Level {
	pub map_num: u8,
	pub geom: Geometry,
	pub state: LevelState,
	pub things: Vec<MapThing>,
}

pub struct Opening {
	pub top: f32,
	pub floor_high: f32,
	pub floor_low: f32,
}

impl Level {
	pub fn load(wad_manager: &WadManager, map_num: u8) -> Result<Self, String> {
		let mut level = Level {
			map_num,
			..Default::default()
		};

		let map_name = construct_map_name(wad_manager.is_doom1, map_num);

		let vertexes_bytes = wad_manager.get_map_data(b"VERTEXES", &map_name)?;
		level.geom.vertices = vertexes_bytes
			.as_chunks::<{ size_of::<MapVertex>() }>()
			.0
			.iter()
			.map(|chunk| {
				let v = unsafe { read_unaligned(chunk.as_ptr() as *const MapVertex) };
				(v.x as f32, v.y as f32)
			})
			.collect();

		let sidedefs_bytes = wad_manager.get_map_data(b"SIDEDEFS", &map_name)?;
		let sides_num = sidedefs_bytes.len() / size_of::<MapSidedef>();
		level.geom.sides.resize_with(sides_num, Side::default);
		level.state.sides.resize_with(sides_num, SideState::default);

		sidedefs_bytes
			.as_chunks::<{ size_of::<MapSidedef>() }>()
			.0
			.iter()
			.enumerate()
			.for_each(|(idx, chunk)| {
				let s = unsafe { read_unaligned(chunk.as_ptr() as *const MapSidedef) };

				level.geom.sides[idx] = Side {
					sector: SectorId(s.sector as usize),
				};
				level.state.sides[idx] = SideState {
					col_offset: s.textureoffset,
					row_offset: s.rowoffset,
					top_tex: None,
					bottom_tex: None,
					mid_tex: None,
					toptexture: s.toptexture,
					midtexture: s.midtexture,
					bottomtexture: s.bottomtexture,
				};
			});

		let sectors_bytes = wad_manager.get_map_data(b"SECTORS\0", &map_name)?;
		level.state.sectors = sectors_bytes
			.as_chunks::<{ size_of::<MapSector>() }>()
			.0
			.iter()
			.enumerate()
			.map(|(idx, chunk)| {
				let s = unsafe { read_unaligned(chunk.as_ptr() as *const MapSector) };

				if s.tag != 0 {
					match level.geom.tag_sectors.get_mut(&s.tag) {
						Some(sectors) => sectors.push(SectorId(idx)),
						None => {
							let _ = level.geom.tag_sectors.insert(s.tag, vec![SectorId(idx)]);
						}
					}
				}

				SectorState {
					floor_h: s.floorheight as f32,
					ceil_h: s.ceilingheight as f32,
					light: s.lightlevel,
					floor_tex: TextureId(0),
					ceil_tex: TextureId(0),
					floorpic: s.floorpic,
					ceilingpic: s.ceilingpic,
					special: s.special,
					tag: s.tag,
					active_mover: None,
				}
			})
			.collect::<Vec<SectorState>>();

		level.geom.movable_sectors.grow(level.state.sectors.len());

		level.geom.movable_sectors.extend(
			level
				.state
				.sectors
				.iter()
				.enumerate()
				.filter(|(_, sector)| sector.tag != 0)
				.map(|(idx, _)| idx),
		);

		let linedefs_bytes = wad_manager.get_map_data(b"LINEDEFS", &map_name)?;
		level.geom.lines = linedefs_bytes
			.as_chunks::<{ size_of::<MapLinedef>() }>()
			.0
			.iter()
			.map(|chunk| {
				let l = unsafe { read_unaligned(chunk.as_ptr() as *const MapLinedef) };

				if l.special != 0 {
					match l.special {
						1 | 26 | 27 | 28 | 31 | 32 | 33 | 34 | 117 | 118
							if l.sidenum[1] != u16::MAX =>
						{
							let sector_id = level.geom.sides[l.sidenum[1] as usize].sector;
							level.geom.movable_sectors.insert(sector_id.0);
						}

						_ => {}
					}
				}

				let (v1_x, v1_y) = level.geom.vertices[l.v1 as usize];
				let (v2_x, v2_y) = level.geom.vertices[l.v2 as usize];

				Line {
					v1: VertexId(l.v1 as usize),
					v2: VertexId(l.v2 as usize),
					flags: LineFlags::from_bits(l.flags).unwrap_or(LineFlags::NONE),
					special: l.special,
					tag: l.tag,
					sides: (
						if l.sidenum[0] == u16::MAX {
							None
						} else {
							Some(SideId(l.sidenum[0] as usize))
						},
						if l.sidenum[1] == u16::MAX {
							None
						} else {
							Some(SideId(l.sidenum[1] as usize))
						},
					),
					delta: (v2_x - v1_x, v2_y - v1_y),
					bbox: AABB {
						min_x: v1_x.min(v2_x),
						max_x: v1_x.max(v2_x),
						min_z: v1_y.min(v2_y),
						max_z: v1_y.max(v2_y),
					},
					slope: get_slope_type(v1_x - v2_x, v1_y - v2_y),
				}
			})
			.collect();

		level
			.state
			.lines
			.resize_with(level.geom.lines.len(), LineState::default);

		let segs_bytes = wad_manager.get_map_data(b"SEGS\0\0\0\0", &map_name)?;
		level.geom.segs = segs_bytes
			.as_chunks::<{ size_of::<MapSegment>() }>()
			.0
			.iter()
			.map(|chunk| unsafe { read_unaligned(chunk.as_ptr() as *const MapSegment) })
			.collect();

		let ssectors_bytes = wad_manager.get_map_data(b"SSECTORS", &map_name)?;
		level.geom.subsectors = ssectors_bytes
			.as_chunks::<{ size_of::<MapSubsector>() }>()
			.0
			.iter()
			.map(|chunk| unsafe { read_unaligned(chunk.as_ptr() as *const MapSubsector) })
			.collect();

		level.geom.subsector_sector = level
			.geom
			.subsectors
			.iter()
			.map(|subsector| {
				let sides = &level.geom.sides;
				let segs = &level.geom.segs;
				let lines = &level.geom.lines;

				let seg = &segs[subsector.firstseg as usize];
				let line = &lines[seg.linedef as usize];
				if seg.side == 0 {
					sides[line.sides.0.unwrap().0].sector
				} else {
					sides[line.sides.1.unwrap().0].sector
				}
			})
			.collect();

		let raw_reject_table = wad_manager.get_map_data(b"REJECT\0\0", &map_name)?;
		level.geom.reject_table =
			if raw_reject_table.is_empty() || raw_reject_table.iter().all(|byte| *byte == 0) {
				RejectTable(None)
			} else {
				RejectTable(Some(raw_reject_table.to_vec()))
			};

		let things_bytes = wad_manager.get_map_data(b"THINGS\0\0", &map_name)?;
		level.things = things_bytes
			.as_chunks::<{ size_of::<MapThing>() }>()
			.0
			.iter()
			.map(|chunk: &[u8; size_of::<MapThing>()]| unsafe {
				read_unaligned(chunk.as_ptr() as *const MapThing)
			})
			.collect();

		let nodes_bytes = wad_manager.get_map_data(b"NODES\0\0\0", &map_name)?;
		level.geom.nodes = nodes_bytes
			.as_chunks::<{ size_of::<MapNode>() }>()
			.0
			.iter()
			.map(|chunk: &[u8; size_of::<MapNode>()]| unsafe {
				read_unaligned(chunk.as_ptr() as *const MapNode)
			})
			.collect();

		let blockmap_bytes = wad_manager.get_map_data(b"BLOCKMAP", &map_name)?;
		let origin_x = i16::from_le_bytes(
			blockmap_bytes
				.get(..2)
				.ok_or_else(|| {
					format!(
						"Invalid origin_x of blockmap in {}",
						String::from_utf8_lossy(&map_name)
					)
				})?
				.try_into()
				.unwrap(),
		);
		let origin_z = i16::from_le_bytes(
			blockmap_bytes
				.get(2..4)
				.ok_or_else(|| {
					format!(
						"Invalid origin_y of blockmap in {}",
						String::from_utf8_lossy(&map_name)
					)
				})?
				.try_into()
				.unwrap(),
		);
		let col_num = u16::from_le_bytes(
			blockmap_bytes
				.get(4..6)
				.ok_or_else(|| {
					format!(
						"Invalid col_num of blockmap in {}",
						String::from_utf8_lossy(&map_name)
					)
				})?
				.try_into()
				.unwrap(),
		) as usize;
		let row_num = u16::from_le_bytes(
			blockmap_bytes
				.get(6..8)
				.ok_or_else(|| {
					format!(
						"Invalid row_num of blockmap in {}",
						String::from_utf8_lossy(&map_name)
					)
				})?
				.try_into()
				.unwrap(),
		) as usize;

		level.geom.blockmap = Blockmap {
			origin_x,
			origin_z,
			col_num,
			row_num,
			blocklists: vec![Vec::new(); col_num * row_num],
		};

		let line_block_pairs: Vec<(usize, LineId)> = level
			.geom
			.lines
			.par_iter()
			.enumerate()
			.flat_map_iter(|(line_idx, line)| {
				let v1 = level.geom.vertices[line.v1.0];
				let v2 = level.geom.vertices[line.v2.0];
				let line_id = LineId(line_idx);

				let block_indices = level.geom.blockmap.get_line_blocklist(line, v1, v2);

				block_indices
					.into_iter()
					.map(move |block_idx| (block_idx, line_id))
			})
			.collect();

		for (block_idx, line_id) in line_block_pairs {
			level.geom.blockmap.blocklists[block_idx].push(line_id);
		}

		Ok(level)
	}

	pub fn get_opening(&self, line_id: LineId) -> Option<Opening> {
		let line = &self.geom.lines[line_id.0];

		let get_sector = |side_id_opt: Option<SideId>| -> Option<&SectorState> {
			let side = &self.geom.sides[side_id_opt?.0];
			Some(&self.state.sectors[side.sector.0])
		};

		let front_sector = get_sector(line.sides.0)?;
		let back_sector = get_sector(line.sides.1)?;

		let top = front_sector.ceil_h.min(back_sector.ceil_h);
		let floor_high = front_sector.floor_h.max(back_sector.floor_h);
		let floor_low = front_sector.floor_h.min(back_sector.floor_h);

		Some(Opening {
			top,
			floor_high,
			floor_low,
		})
	}

	pub fn get_other_sector(&self, line_id: LineId, sector_id: SectorId) -> Option<SectorId> {
		let line = &self.geom.lines[line_id.0];

		match line.sides {
			(Some(front_side_id), Some(back_side_id)) => {
				let front_sector = self.geom.sides[front_side_id.0].sector;
				let back_sector = self.geom.sides[back_side_id.0].sector;

				if front_sector == sector_id {
					Some(back_sector)
				} else {
					Some(front_sector)
				}
			}
			_ => None,
		}
	}
}

pub fn to_u64(name_bytes: &[u8]) -> u64 {
	let mut buf = [0u8; 8];

	for i in 0..8 {
		if i >= name_bytes.len() || name_bytes[i] == 0 {
			break;
		}
		buf[i] = name_bytes[i].to_ascii_uppercase();
	}

	u64::from_le_bytes(buf)
}

pub fn construct_map_name(is_doom1: bool, num: u8) -> [u8; 8] {
	if is_doom1 {
		let map_idx = num - 1;
		let episode = (map_idx / 9) + 1 + b'0';
		let map_num = (map_idx % 9) + 1 + b'0';
		[b'E', episode, b'M', map_num, 0, 0, 0, 0]
	} else {
		let tens = num / 10 + b'0';
		let ones = num % 10 + b'0';
		[b'M', b'A', b'P', tens, ones, 0, 0, 0]
	}
}

impl RejectTable {
	pub fn is_rejected(
		&self,
		src_sector: SectorId,
		target_sector: SectorId,
		num_sectors: usize,
	) -> bool {
		let Some(ref reject) = self.0 else {
			return false;
		};

		if src_sector.0 >= num_sectors || target_sector.0 >= num_sectors {
			return false;
		}

		let bit_index = src_sector.0 * num_sectors + target_sector.0;
		let byte_index = bit_index >> 3;
		let bit_offset = bit_index & 0b111;

		if byte_index >= reject.len() {
			return false;
		}

		(reject[byte_index] & (1 << bit_offset)) != 0
	}
}

pub const MAPBLOCKSIZE: f32 = 128.0;

impl Blockmap {
	fn get_line_blocklist(&self, line: &Line, v1: (f32, f32), v2: (f32, f32)) -> Vec<usize> {
		let mut result = Vec::new();
		let origin_x = self.origin_x as f32;
		let origin_z = self.origin_z as f32;

		let min_block_x = ((line.bbox.min_x - origin_x) / MAPBLOCKSIZE).floor() as i32;
		let max_block_x = ((line.bbox.max_x - origin_x) / MAPBLOCKSIZE).floor() as i32;
		let min_block_z = ((line.bbox.min_z - origin_z) / MAPBLOCKSIZE).floor() as i32;
		let max_block_z = ((line.bbox.max_z - origin_z) / MAPBLOCKSIZE).floor() as i32;

		if max_block_x < 0
			|| min_block_x >= self.col_num as i32
			|| max_block_z < 0
			|| min_block_z >= self.row_num as i32
		{
			return result;
		}

		let start_col = min_block_x.clamp(0, self.col_num as i32 - 1) as usize;
		let end_col = max_block_x.clamp(0, self.col_num as i32 - 1) as usize;
		let start_row = min_block_z.clamp(0, self.row_num as i32 - 1) as usize;
		let end_row = max_block_z.clamp(0, self.row_num as i32 - 1) as usize;

		for row in start_row..=end_row {
			for col in start_col..=end_col {
				if start_col == end_col || start_row == end_row {
					result.push(row * self.col_num + col);
				} else {
					let block_aabb = AABB {
						min_x: origin_x + (col as f32) * MAPBLOCKSIZE,
						max_x: origin_x + ((col + 1) as f32) * MAPBLOCKSIZE,
						min_z: origin_z + (row as f32) * MAPBLOCKSIZE,
						max_z: origin_z + ((row + 1) as f32) * MAPBLOCKSIZE,
					};

					if block_aabb.intersects_line(&line.bbox, v1, v2) {
						result.push(row * self.col_num + col);
					}
				}
			}
		}

		result
	}

	pub fn world_to_grid(&self, x: f32, z: f32) -> (usize, usize) {
		let col = ((x - self.origin_x as f32) / MAPBLOCKSIZE).floor() as i32;
		let row = ((z - self.origin_z as f32) / MAPBLOCKSIZE).floor() as i32;

		let safe_col = col.clamp(0, (self.col_num - 1) as i32) as usize;
		let safe_row = row.clamp(0, (self.row_num - 1) as i32) as usize;

		(safe_col, safe_row)
	}

	pub fn for_each_line_in_aabb<F>(&self, bbox: &AABB, mut callback: F) -> bool
	where
		F: FnMut(LineId) -> bool,
	{
		let (min_col, min_row) = self.world_to_grid(bbox.min_x, bbox.min_z);
		let (max_col, max_row) = self.world_to_grid(bbox.max_x, bbox.max_z);

		for row in min_row..=max_row {
			for col in min_col..=max_col {
				let idx = row * self.col_num + col;
				for &line_idx in &self.blocklists[idx] {
					if !callback(line_idx) {
						return false;
					}
				}
			}
		}
		true
	}
}

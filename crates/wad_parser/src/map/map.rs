use crate::{AABB, WadManager};
use std::ptr::read_unaligned;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapVertex
{
	pub x: i16,
	pub y: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapSidedef
{
	pub textureoffset: i16,
	pub rowoffset: i16,
  	pub toptexture: [u8; 8],
  	pub bottomtexture: [u8; 8],
  	pub midtexture: [u8; 8],
	pub sector: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapLinedef
{
	pub v1: i16,
	pub v2: i16,
	pub flags: i16,
	pub special: i16,
	pub tag: i16,
	pub sidenum: [u16; 2]	
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapSector
{
	pub floorheight: i16,
	pub ceilingheight: i16,
  	pub floorpic: [u8; 8],
  	pub ceilingpic:[u8; 8],
	pub lightlevel: i16,
	pub special: i16,
	pub tag: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapSubsector
{
	pub numsegs: i16,
	pub firstseg: u16	
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapSegment
{
	pub v1: i16,
	pub v2: i16,
	pub angle: i16,
	pub linedef: u16,
	pub side: i16,
	pub offset: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapNode
{
	pub x: i16,
	pub y: i16,
	pub dx: i16,
	pub dy: i16,
	pub bbox: [[i16; 4]; 2],
	pub children: [u16; 2],
}

#[repr(C)] 
#[derive(Debug, Clone, Copy)]
pub struct MapThing
{
	pub x: i16,
	pub y: i16,
	pub angle: i16,
	pub type_: i16,
	pub options: i16,
}

#[derive(Debug, Clone, Default)]
pub struct Blockmap {
	pub origin_x: i16,
	pub origin_y: i16,
	pub col_num: usize,
	pub row_num: usize,
	pub blocklists: Vec<Vec<usize>>
}

#[derive(Debug, Clone, Default)]
pub struct RejectTable(pub Option<Vec<u8>>);

#[derive(Debug, Default)]
pub struct DoomMap {
	pub map_num: u8,
    pub vertices: Vec<MapVertex>,
    pub linedefs: Vec<MapLinedef>,
    pub sidedefs: Vec<MapSidedef>,
    pub sectors: Vec<MapSector>,
    pub things: Vec<MapThing>,
	pub subsectors: Vec<MapSubsector>,
	pub segs: Vec<MapSegment>,
	pub nodes: Vec<MapNode>,
	pub reject_table: RejectTable,
	pub blockmap: Blockmap
}

impl DoomMap {
    pub fn from_wad(wad_manager: &WadManager, map_num: u8) -> Result<Self, String> {
		let mut map = DoomMap::default();
		map.map_num = map_num;

		let map_name = construct_map_name(wad_manager.is_doom1, map_num);

    	let vertexes_bytes = wad_manager.get_map_data(b"VERTEXES", &map_name)?;
		map.vertices = vertexes_bytes
    		.chunks_exact(size_of::<MapVertex>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapVertex) }
    		})
    		.collect();

		let linedefs_bytes = wad_manager.get_map_data(b"LINEDEFS", &map_name)?;
		map.linedefs = linedefs_bytes
    		.chunks_exact(size_of::<MapLinedef>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapLinedef) }
    		})
    		.collect();

		let sidedefs_bytes = wad_manager.get_map_data(b"SIDEDEFS", &map_name)?;
		map.sidedefs = sidedefs_bytes
    		.chunks_exact(size_of::<MapSidedef>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSidedef) }
    		})
    		.collect();

		let sectors_bytes = wad_manager.get_map_data(b"SECTORS\0", &map_name)?;
		map.sectors = sectors_bytes
    		.chunks_exact(size_of::<MapSector>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSector) }
    		})
    		.collect();

		let raw_reject_table = wad_manager.get_map_data(b"REJECT\0\0", &map_name)?;
		map.reject_table = if raw_reject_table.is_empty() || raw_reject_table.iter().all(|byte| *byte == 0) {
			RejectTable(None)
		} else {
			RejectTable(Some(raw_reject_table.to_vec()))
		};

		let things_bytes = wad_manager.get_map_data(b"THINGS\0\0", &map_name)?;
		map.things = things_bytes
    		.chunks_exact(size_of::<MapThing>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapThing) }
    		})
    		.collect();

		let ssectors_bytes = wad_manager.get_map_data(b"SSECTORS", &map_name)?;
		map.subsectors = ssectors_bytes
    		.chunks_exact(size_of::<MapSubsector>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSubsector) }
    		})
    		.collect();

		let segs_bytes = wad_manager.get_map_data(b"SEGS\0\0\0\0", &map_name)?;
		map.segs = segs_bytes
    		.chunks_exact(size_of::<MapSegment>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSegment) }
    		})
    		.collect();

		let nodes_bytes = wad_manager.get_map_data(b"NODES\0\0\0", &map_name)?;
		map.nodes = nodes_bytes
    		.chunks_exact(size_of::<MapNode>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapNode) }
    		})
    		.collect();

		
		let blockmap_bytes = wad_manager.get_map_data(b"BLOCKMAP", &map_name)?;
		let origin_x = i16::from_le_bytes(blockmap_bytes
			.get(..2)
			.ok_or_else(|| format!("Invalid origin_x of blockmap in {}", String::from_utf8_lossy(&map_name)))?
			.try_into().unwrap()
		);
		let origin_y = i16::from_le_bytes(blockmap_bytes
			.get(2..4)
			.ok_or_else(|| format!("Invalid origin_y of blockmap in {}", String::from_utf8_lossy(&map_name)))?
			.try_into().unwrap()
		);
		let col_num = u16::from_le_bytes(blockmap_bytes
			.get(4..6)
			.ok_or_else(|| format!("Invalid col_num of blockmap in {}", String::from_utf8_lossy(&map_name)))?
			.try_into().unwrap()
		) as usize;
		let row_num = u16::from_le_bytes(blockmap_bytes
			.get(6..8)
			.ok_or_else(|| format!("Invalid row_num of blockmap in {}", String::from_utf8_lossy(&map_name)))?
			.try_into().unwrap()
		) as usize;

		let offsets: Vec<usize> = blockmap_bytes
			.get(8..8 + 2 * (col_num * row_num))
			.ok_or_else(|| format!("Invalid offsets of blockmap in {}", String::from_utf8_lossy(&map_name)))?
			.chunks_exact(2)
			.map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()) as usize)
			.collect();

		let mut blocklists = vec![Vec::new(); col_num * row_num];
		for (block_idx, &u16_offset) in offsets.iter().enumerate() {
		    let mut byte_offset = u16_offset * 2;

		    if byte_offset >= blockmap_bytes.len() {
		        continue;
		    }
		
		    if byte_offset + 2 <= blockmap_bytes.len() {
		        let start_zero = u16::from_le_bytes(blockmap_bytes[byte_offset..byte_offset + 2].try_into().unwrap());
		        if start_zero == 0 {
		            byte_offset += 2;
		        }
		    }
		
		    while byte_offset + 2 <= blockmap_bytes.len() {
		        let line_idx = u16::from_le_bytes(blockmap_bytes[byte_offset..byte_offset + 2].try_into().unwrap());
			
		        if line_idx == u16::MAX {
		            break;
		        }
			
		        blocklists[block_idx].push(line_idx as usize);
		        byte_offset += 2;
		    }
		}

		map.blockmap = Blockmap {
			origin_x,
			origin_y,
			col_num,
			row_num,
			blocklists
		};
		

        Ok(map)
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
	pub fn is_rejected(&self, src_sector: usize, target_sector: usize, num_sectors: usize) -> bool {
        let Some(ref reject) = self.0 else {
            return false;
        };

        if src_sector >= num_sectors || target_sector >= num_sectors {
            return false;
        }

        let bit_index = src_sector * num_sectors + target_sector;
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
	pub fn world_to_grid(&self, x: f32, y: f32) -> (usize, usize) {
        let col = ((x - self.origin_x as f32) / MAPBLOCKSIZE).floor() as i32;
        let row = ((y - self.origin_y as f32) / MAPBLOCKSIZE).floor() as i32;
		
		let safe_col = col.clamp(0, (self.col_num - 1) as i32) as usize;
    	let safe_row = row.clamp(0, (self.row_num - 1) as i32) as usize;

    	(safe_col, safe_row)
    }

    pub fn for_each_line_in_aabb<F>(&self, bbox: &AABB, mut callback: F) -> bool
    where
        F: FnMut(usize) -> bool,
    {
        let (min_col, min_row) = self.world_to_grid(bbox.min_x, bbox.min_y);
        let (max_col, max_row) = self.world_to_grid(bbox.max_x, bbox.max_y);

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

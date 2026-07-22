use crate::WadManager;
use std::ptr::read_unaligned;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Sector {
	pub props: MapSector,
	pub sound_traversed: u32,
	pub lines: Vec<usize>
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

#[derive(Debug, Default)]
pub struct DoomMap {
	pub map_num: u8,
    pub vertices: Vec<MapVertex>,
    pub linedefs: Vec<MapLinedef>,
    pub sidedefs: Vec<MapSidedef>,
    pub sectors: Vec<Sector>,
    pub things: Vec<MapThing>,
	pub subsectors: Vec<MapSubsector>,
	pub segs: Vec<MapSegment>,
	pub nodes: Vec<MapNode>
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
    		    unsafe { 
					let props = read_unaligned(chunk.as_ptr() as *const MapSector);
					Sector { props, sound_traversed: u32::MAX, lines: Vec::with_capacity(5) }
				}
    		})
    		.collect();

		for (line_idx, line) in map.linedefs.iter().enumerate() {
		    if line.sidenum[0] != u16::MAX {
		        let front_sector = map.sidedefs[line.sidenum[0] as usize].sector;
		        map.sectors[front_sector as usize].lines.push(line_idx);
		    }
		    if line.sidenum[1] != u16::MAX {
		        let back_sector = map.sidedefs[line.sidenum[1] as usize].sector;
		        map.sectors[back_sector as usize].lines.push(line_idx);
		    }
		}

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

        Ok(map)
    }
}

pub fn to_u64(name_bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];

    for i in 0..8 {
        if i >= name_bytes.len() || name_bytes[i] == 0 {
            break;
        }
        buf[i] = name_bytes[i];
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MapVertex
{
	pub(crate) x: i16,
	pub(crate) y: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MapSidedef
{
	pub(crate) textureoffset: i16,
	pub(crate) rowoffset: i16,
  	pub(crate) toptexture: [u8; 8],
  	pub(crate) bottomtexture: [u8; 8],
  	pub(crate) midtexture: [u8; 8],
	pub(crate) sector: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MapLinedef
{
	pub(crate) v1: u16,
	pub(crate) v2: u16,
	pub(crate) flags: u16,
	pub(crate) special: u16,
	pub(crate) tag: u16,
	pub(crate) sidenum: [u16; 2]	
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MapSector
{
	pub(crate) floorheight: i16,
	pub(crate) ceilingheight: i16,
  	pub(crate) floorpic: [u8; 8],
  	pub(crate) ceilingpic:[u8; 8],
	pub(crate) lightlevel: u16,
	pub(crate) special: u16,
	pub(crate) tag: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapSubsector
{
	pub numsegs: u16,
	pub firstseg: u16	
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapSegment
{
	pub v1: u16,
	pub v2: u16,
	pub angle: i16,
	pub linedef: u16,
	pub side: u16,
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
	pub type_: u16,
	pub options: u16,
}

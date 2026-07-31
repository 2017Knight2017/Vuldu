use hecs::Entity;
use wad_parser::{DoomMap, MapLinedef, MapSector};

#[derive(Debug, Clone)]
pub struct DynMap {
	pub sectors: Vec<DynSector>,
	pub linedefs: Vec<DynLinedef>,
	pub valid_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct DynLinedef {
	pub props: MapLinedef,
	pub valid_count: u32, 
}

#[derive(Debug, Clone)]
pub struct DynSector {
    pub props: MapSector,
    pub sound_traversed: u32,
    pub lines: Vec<usize>,
    pub sound_target: Option<Entity>,
}

impl From<&DoomMap> for DynMap {
	fn from(value: &DoomMap) -> Self {
		let sectors: Vec<DynSector> = value.sectors
        	.iter()
        	.map(|&props| DynSector {
        	    props,
        	    sound_traversed: u32::MAX,
        	    lines: Vec::with_capacity(5),
        	    sound_target: None,
        	})
        	.collect();

		let linedefs: Vec<DynLinedef> = value.linedefs
			.iter()
			.map(|&props| DynLinedef {
				props,
				valid_count: 0
			})
			.collect();

		DynMap { sectors, linedefs, valid_count: 0 }
	}
}

use std::path::Path;
use rustc_hash::FxHashMap;
use crate::{Lump, Wad};

pub struct WadManager {
    pub is_doom1: bool,
    pub wads: Vec<Wad>,
    pub map_directory: FxHashMap<[u8; 8], [Lump; 10]>,
    pub directory: FxHashMap<[u8; 8], Lump>,
    pub palettes: Vec<Option<Lump>>,
    pub colormaps: Vec<Option<Lump>>,
}

impl WadManager {
    pub fn new() -> Self {
        Self {
            is_doom1: false,
            wads: Vec::new(),
            map_directory: FxHashMap::default(),
            directory: FxHashMap::default(),
            palettes: Vec::new(),
            colormaps: Vec::new(),
        }
    }

    pub fn add_wad<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path_ref = path.as_ref();
        
        if let Some(file_name) = path_ref.file_name().and_then(|os_str| os_str.to_str()) {
            if file_name.to_uppercase() == "DOOM.WAD" {
                self.is_doom1 = true;
            }
        }

        let (wad, parsed_lumps, parsed_maps) = Wad::open(path)?;
        
        let wad_index = self.wads.len();
        self.wads.push(wad);

        for parsed in parsed_lumps {
            let mut clean_name = parsed.name;
            clean_name.make_ascii_uppercase();

            let lump = Lump {
                wad_index,
                offset: parsed.offset,
                size: parsed.size,
            };

            if clean_name == *b"PLAYPAL\0" {
                self.palettes.push(Some(lump));
            } else if clean_name == *b"COLORMAP" {
                self.colormaps.push(Some(lump));
            } else {
                self.directory.insert(clean_name, lump);
            }
        }

        if self.palettes.len() < self.wads.len() {
            self.palettes.push(None);
        }
        if self.colormaps.len() < self.wads.len() {
            self.colormaps.push(None);
        }

        for parsed_map in parsed_maps {
            let map_lumps: [Lump; 10] = std::array::from_fn(|lump_idx| {
                let parsed_lump = &parsed_map.1[lump_idx];
                Lump {
                    wad_index,
                    offset: parsed_lump.offset,
                    size: parsed_lump.size,
                }
            });

            let mut clean_map_name = parsed_map.0;
            clean_map_name.make_ascii_uppercase();

            self.map_directory.insert(clean_map_name, map_lumps);
        }

        Ok(())
    }

	pub fn get_data(&self, lump_name: &[u8]) -> Result<&[u8], String> {
        let mut normalized_name = [0u8; 8];

        for (dest, &src) in normalized_name.iter_mut().zip(lump_name.iter()) {
            *dest = src.to_ascii_uppercase();
        }

		let lump = self.directory.get(&normalized_name)
            .ok_or_else(|| format!("Required lump '{}' is missing", String::from_utf8_lossy(lump_name)))?;
            
        let wad = self.wads.get(lump.wad_index)
            .ok_or_else(|| format!("Internal Error: WAD index {} is invalid", lump.wad_index))?;
            
        let start = lump.offset;
        let end = start + lump.size;
        
        wad.data.get(start..end)
            .ok_or_else(|| format!("Lump '{}' has out-of-bounds data slice", String::from_utf8_lossy(lump_name)))
    }

    pub fn get_map_data(&self, lump_name: &[u8; 8], map_name: &[u8; 8]) -> Result<&[u8], String> {
        let lump_idx: usize = match lump_name {
            b"THINGS\0\0" => 0,
            b"LINEDEFS" => 1,
            b"SIDEDEFS" => 2,
            b"VERTEXES" => 3,
            b"SEGS\0\0\0\0" => 4,
            b"SSECTORS" => 5,
            b"NODES\0\0\0" => 6,
            b"SECTORS\0" => 7,
            b"REJECT\0\0" => 8,
            b"BLOCKMAP" => 9,
            _ => return Err(format!("There is no maplump with name '{}'", String::from_utf8_lossy(lump_name)))
        };

        if let Some(map) = self.map_directory.get(map_name) {
            let lump = map[lump_idx];
            let wad = &self.wads[lump.wad_index];

            let start = lump.offset;
            let end = start + lump.size;
            
            wad.data.get(start..end)
                .ok_or_else(|| format!("Lump '{}' has out-of-bounds data slice", String::from_utf8_lossy(lump_name)))
        } else {
            Err(format!("There is no map named '{}'", String::from_utf8_lossy(map_name)))
        }
    }

	pub fn get_palettes(&self, map_name: &[u8; 8]) -> Result<Vec<f32>, String> {
        let mut all_palettes_data = Vec::with_capacity(14 * 256 * 4);

        let map_wad_index = self.map_directory[map_name][0].wad_index;
        let pal_wad_index = match self.colormaps[map_wad_index] {
            Some(_) => map_wad_index,
            None => 0, 
        };

        let pal_lump = self.palettes[pal_wad_index].unwrap();
		let pal_data = self.wads[pal_wad_index].data.get(pal_lump.offset..pal_lump.offset + pal_lump.size)
            .ok_or_else(|| format!("PLAYPAL lump is not found!"))?;
        
        for rgb in pal_data.chunks_exact(3).take(14 * 256) {
            all_palettes_data.push(rgb[0] as f32 / 255.0);
            all_palettes_data.push(rgb[1] as f32 / 255.0);
            all_palettes_data.push(rgb[2] as f32 / 255.0);
            all_palettes_data.push(1.0);
        }

        Ok(all_palettes_data)
    }

    pub fn get_colormap(&self, map_name: &[u8; 8]) -> Result<&[u8], String> {
        let map_wad_index = self.map_directory[map_name][0].wad_index;
        let clm_wad_index = match self.colormaps[map_wad_index] {
            Some(_) => map_wad_index,
            None => 0, 
        };

        let clm_lump = self.colormaps[clm_wad_index].unwrap();
        let clm_data = self.wads[clm_wad_index].data.get(clm_lump.offset..clm_lump.offset + clm_lump.size)
            .ok_or_else(|| format!("COLORMAP lump is not found!"))?;
		
        Ok(clm_data)
    }
}

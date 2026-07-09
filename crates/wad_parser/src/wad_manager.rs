use std::collections::HashMap;
use std::path::Path;
use crate::{Lump, Wad};

pub struct WadManager {
    pub is_doom1: bool,
    pub wads: Vec<Wad>,
    pub directory: HashMap<String, Lump>,
}

impl WadManager {
    pub fn new() -> Self {
        Self {
            is_doom1: false,
            wads: Vec::new(),
            directory: HashMap::new(),
        }
    }

    pub fn add_wad<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path_ref = path.as_ref();
        
        if let Some(file_name) = path_ref.file_name().and_then(|os_str| os_str.to_str()) {
            if file_name.to_lowercase() == "doom.wad" {
                self.is_doom1 = true;
            }
        }

        let (wad, parsed_lumps) = Wad::open(path)?;
        
        let wad_index = self.wads.len();
        self.wads.push(wad);

        for parsed in parsed_lumps {
            let lump = Lump {
                wad_index,
                offset: parsed.offset,
                size: parsed.size,
            };
            
            self.directory.insert(parsed.name, lump);
        }

        Ok(())
    }

	pub fn get_data(&self, lumpname: &str) -> Result<&[u8], String> {
		let lump = self.directory.get(lumpname)
            .ok_or_else(|| format!("Required lump '{}' is missing", lumpname))?;
            
        let wad = self.wads.get(lump.wad_index)
            .ok_or_else(|| format!("Internal Error: WAD index {} is invalid", lump.wad_index))?;
            
        let start = lump.offset;
        let end = start + lump.size;
        
        wad.data.get(start..end)
            .ok_or_else(|| format!("Lump '{}' has out-of-bounds data slice", lumpname))
    }

	pub fn get_palettes(&self) -> Result<Vec<f32>, String> {
        let mut all_palettes_data = vec![0.0f32; 14 * 256 * 4];

		let playpal_lump = self.get_data("PLAYPAL")?;

        for palette_idx in 0..14 {
            for color_idx in 0..256 {
                let global_color_offset = palette_idx * 256 * 3 + color_idx * 3;
                let target_offset = (palette_idx * 256 + color_idx) * 4;
            
                all_palettes_data[target_offset + 0] = playpal_lump[global_color_offset + 0] as f32 / 255.0;
                all_palettes_data[target_offset + 1] = playpal_lump[global_color_offset + 1] as f32 / 255.0;
                all_palettes_data[target_offset + 2] = playpal_lump[global_color_offset + 2] as f32 / 255.0;
                all_palettes_data[target_offset + 3] = 1.0; 
            }
        }

        Ok(all_palettes_data)
    }
}
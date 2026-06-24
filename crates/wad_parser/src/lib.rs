pub mod map;
pub mod textures;

use std::collections::HashMap;
use std::path::Path;
use std::fs::read;
use std::array::TryFromSliceError;

#[derive(Clone, Copy, Debug)]
pub struct Lump {
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug)] 
pub struct Wad {
    data: Vec<u8>,
    pub directory: HashMap<String, Lump>,
    pub dir_offset: usize
}

impl Wad {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
		let data = read(path).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err("File is to small for a WAD".to_string());
        }

        let header = std::str::from_utf8(&data[0..4]).map_err(|e| e.to_string())?;
        if header != "IWAD" && header != "PWAD" {
            return Err(format!("Invalid WAD format: {}, which is not IWAD or PWAD", header));
        }

        let num_lumps = u32::from_le_bytes(data[4..8].try_into().map_err(|_| "num_lumps is invalid")?) as usize;
        let dir_offset = u32::from_le_bytes(data[8..12].try_into().map_err(|_| "dir_offset is invalid")?) as usize;

        let mut directory = HashMap::new();
        let mut current_dir_pos = dir_offset;

        while current_dir_pos < dir_offset + num_lumps * 16 {
            if current_dir_pos + 16 > data.len() {
                return Err(format!("Directory is damaged on lump {}", current_dir_pos / 16));
            }

            let lump_offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4]
                .try_into()
                .map_err(|e: TryFromSliceError| e.to_string())?) as usize;
            let lump_size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8]
                .try_into()
                .map_err(|e: TryFromSliceError| e.to_string())?) as usize;
            
            let name_bytes = &data[current_dir_pos+8..current_dir_pos+16];
            let name = String::from_utf8_lossy(name_bytes)
                .trim_matches('\0')
                .to_uppercase();

            current_dir_pos += 16;

            if &name[0..3] == "MAP" || (&name[0..1] == "E" && &name[2..3] == "M") {
                for _ in 0..10 {
                    let maplump_offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4]
                        .try_into()
                        .map_err(|e: TryFromSliceError| e.to_string())?) as usize;
                    let maplump_size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8]
                        .try_into()
                        .map_err(|e: TryFromSliceError| e.to_string())?) as usize;
                    
                    let maplump_name_bytes = &data[current_dir_pos+8..current_dir_pos+16];
                    let maplump_name = String::from_utf8_lossy(maplump_name_bytes)
                        .trim_matches('\0')
                        .to_uppercase();

                    directory.insert(maplump_name + "_" + &name, Lump { offset: maplump_offset, size: maplump_size });

                    current_dir_pos += 16;
                }
            } else {
                directory.insert(name, Lump { offset: lump_offset, size: lump_size });
            }

        }

        Ok(Wad { data, directory, dir_offset })
    }

    pub fn get_data_by_lumpname(&self, name: &str) -> Option<&[u8]> {
        let lump = self.directory.get(&name.to_uppercase())?;
        let start = lump.offset;
        let end = start + lump.size;
        
        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    pub fn get_palettes(&self) -> Option<Vec<f32>> {
        let mut all_palettes_data = vec![0.0f32; 14 * 256 * 4];
                    
        let playpal_lump = self.get_data_by_lumpname("PLAYPAL").expect("No palette!");

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

        Some(all_palettes_data)
    }
}
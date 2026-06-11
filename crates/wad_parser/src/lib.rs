pub mod sprite;

use std::collections::HashMap;
use std::path::Path;
use std::fs::read;

#[derive(Clone, Copy)]
pub struct Lump {
    pub offset: usize,
    pub size: usize,
}

pub struct Wad {
    data: Vec<u8>,
    pub directory: HashMap<String, Lump>,
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

        for i in 0..num_lumps {
            if current_dir_pos + 16 > data.len() {
                return Err(format!("Directory is damaged on lump {}", i));
            }

            let lump_offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4].try_into().unwrap()) as usize;
            let lump_size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8].try_into().unwrap()) as usize;
            
            let name_bytes = &data[current_dir_pos+8..current_dir_pos+16];
            let name = String::from_utf8_lossy(name_bytes)
                .trim_matches('\0')
                .to_uppercase();

            directory.insert(name, Lump { offset: lump_offset, size: lump_size });
            current_dir_pos += 16;
        }

        Ok(Wad { data, directory })
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
}
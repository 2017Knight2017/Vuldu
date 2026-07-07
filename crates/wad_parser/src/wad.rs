use std::fs::read;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct Lump {
    pub wad_index: usize,
    pub offset: usize,
    pub size: usize,
}

pub struct Wad {
    pub data: Vec<u8>,
	pub dir_offset: usize,
	pub num_lumps: usize,
}

pub struct ParsedLump {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

impl Wad {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<(Self, Vec<ParsedLump>), String> {
        let data = read(path).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err("File is too small for a WAD".to_string());
        }

        let header = std::str::from_utf8(&data[0..4]).map_err(|e| e.to_string())?;
        if header != "IWAD" && header != "PWAD" {
            return Err(format!("Invalid WAD format: {}, which is not IWAD or PWAD", header));
        }

        let num_lumps = u32::from_le_bytes(data[4..8].try_into().map_err(|_| "num_lumps is invalid")?) as usize;
        let dir_offset = u32::from_le_bytes(data[8..12].try_into().map_err(|_| "dir_offset is invalid")?) as usize;

        let mut parsed_lumps = Vec::with_capacity(num_lumps);
        let mut current_dir_pos = dir_offset;

        let dir_end = dir_offset + num_lumps * 16;

        while current_dir_pos < dir_end {
            if current_dir_pos + 16 > data.len() {
                break;
            }

            let lump_offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4].try_into().unwrap()) as usize;
            let lump_size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8].try_into().unwrap()) as usize;
            
            let name_bytes = &data[current_dir_pos+8..current_dir_pos+16];
            let name = String::from_utf8_lossy(name_bytes).trim_matches('\0').to_uppercase();

            current_dir_pos += 16;

            let is_map = name.starts_with("MAP") || (name.starts_with('E') && name.chars().nth(2) == Some('M'));
            
            if is_map {
                parsed_lumps.push(ParsedLump { name: name.clone(), offset: lump_offset, size: lump_size });

                for _ in 0..10 {
                    if current_dir_pos + 16 > data.len() { break; }

                    let maplump_offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4].try_into().unwrap()) as usize;
                    let maplump_size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8].try_into().unwrap()) as usize;
                    
                    let maplump_name_bytes = &data[current_dir_pos+8..current_dir_pos+16];
                    let maplump_name = String::from_utf8_lossy(maplump_name_bytes).trim_matches('\0').to_uppercase();

                    parsed_lumps.push(ParsedLump {
                        name: format!("{}_{}", maplump_name, name),
                        offset: maplump_offset,
                        size: maplump_size,
                    });

                    current_dir_pos += 16;
                }
            } else {
                parsed_lumps.push(ParsedLump { name, offset: lump_offset, size: lump_size });
            }
        }

        Ok((Wad { data, dir_offset, num_lumps }, parsed_lumps))
    }
}
use std::fs::read;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Clone, Copy, Default)]
pub struct ParsedLump {
    pub name: [u8; 8],
    pub offset: usize,
    pub size: usize,
}

type MapLumps = ([u8; 8], [ParsedLump; 10]);

impl Wad {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<(Self, Vec<ParsedLump>, Vec<MapLumps>), String> {
        let data = read(&path).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err(format!("File '{}' is under 12 bytes long", path.as_ref().file_name().unwrap().display()));
        }

        let header = std::str::from_utf8(&data[0..4]).map_err(|e| e.to_string())?;
        if header != "IWAD" && header != "PWAD" {
            return Err(format!("Invalid WAD format: {}, which is not IWAD or PWAD", header));
        }

        let num_lumps = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let dir_offset = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;

        let mut parsed_lumps = Vec::with_capacity(num_lumps);
        let mut maps: Vec<MapLumps> = Vec::new();
        
        let mut current_dir_pos = dir_offset;
        let dir_end = dir_offset + num_lumps * 16;

        while current_dir_pos < dir_end {
            if current_dir_pos + 16 > data.len() {
                break;
            }

            let offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4].try_into().unwrap()) as usize;
            let size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8].try_into().unwrap()) as usize;            
            let name: [u8; 8] = data[current_dir_pos+8..current_dir_pos+16].try_into().unwrap();

            parsed_lumps.push(ParsedLump { name, offset, size });

            current_dir_pos += 16;
            
            let is_map = name.starts_with(b"MAP") || (name[0] == b'E' && name[2] == b'M');
            if !is_map { continue; }

            let mut map: MapLumps = (name, [ParsedLump::default(); 10]);

            for i in 0..10 {
                if current_dir_pos + 16 > data.len() { break; }

                let offset = u32::from_le_bytes(data[current_dir_pos..current_dir_pos+4].try_into().unwrap()) as usize;
                let size = u32::from_le_bytes(data[current_dir_pos+4..current_dir_pos+8].try_into().unwrap()) as usize;
                let name: [u8; 8] = data[current_dir_pos+8..current_dir_pos+16].try_into().unwrap();
                
                map.1[i] = ParsedLump { name, offset, size };

                current_dir_pos += 16;
            }

            maps.push(map);
        }

        //println!("{}, {}, {}, {}, {}, {}", stcfncnt, stcnt, mcnt, wilvcnt, wicnt, amcnt);
        // Doom    : 64, 88, 46, 36, 100, 11
        // Doom 2  : 64, 88, 45, 32, 36, 11
        // TNT     : 64, 88, 45, 32, 36, 11
        // Plutonia: 64, 88, 45, 32, 36, 11
        Ok((Wad { data, dir_offset, num_lumps }, parsed_lumps, maps))
    }
}

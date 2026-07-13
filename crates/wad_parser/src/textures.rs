use crate::*;
use phf::{Map, phf_map};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ptr::read_unaligned;
use std::mem::size_of;

const MAX_SKY: usize = 16;

#[repr(C)]
#[derive(Clone, Copy)]
struct MapPatch {
    originx: i16,
    originy: i16,
    patch: i16,
	stepdir: i16,
    colormap: i16,
}

#[repr(C)]
pub struct MapTexture {
    name: [u8; 8],
    masked: bool,
    width: i16,
    height: i16,
    patchcount: i16,
    patches: Vec<MapPatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoomPicture {
	pub raw_pixels: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub left_offset: i16,
	pub top_offset: i16,
} 

pub static SPRITE_NAMES: Map<i16, Option<&'static str>> = phf_map! {
	1i16 => Some("PLAY"),
	2i16 => Some("PLAY"),
	3i16 => Some("PLAY"),
	4i16 => Some("PLAY"),
	5i16 => Some("BKEY"),
	6i16 => Some("YKEY"),
	7i16 => Some("SPID"),
	8i16 => Some("BPAK"),
	9i16 => Some("SPOS"),
	10i16 => Some("PLAY"),
	11i16 => None,
	12i16 => Some("PLAY"),
	13i16 => Some("RKEY"),
	14i16 => None,
	15i16 => Some("PLAY"),
	16i16 => Some("CYBR"),
	17i16 => Some("CELP"),
	18i16 => Some("POSS"),
	19i16 => Some("SPOS"),
	20i16 => Some("TROO"),
	21i16 => Some("SARG"),
	22i16 => Some("HEAD"),
	23i16 => Some("SKUL"),
	24i16 => Some("POL5"),
	25i16 => Some("POL1"),
	26i16 => Some("POL6"),
	27i16 => Some("POL4"),
	28i16 => Some("POL2"),
	29i16 => Some("POL3"),
	30i16 => Some("COL1"),
	31i16 => Some("COL2"),
	32i16 => Some("COL3"),
	33i16 => Some("COL4"),
	34i16 => Some("CAND"),
	35i16 => Some("CBRA"),
	36i16 => Some("COL5"),
	37i16 => Some("COL6"),
	38i16 => Some("RSKU"),
	39i16 => Some("YSKU"),
	40i16 => Some("BSKU"),
	41i16 => Some("CEYE"),
	42i16 => Some("FSKU"),
	43i16 => Some("TRE1"),
	44i16 => Some("TBLU"),
	45i16 => Some("TGRN"),
	46i16 => Some("TRED"),
	47i16 => Some("SMIT"),
	48i16 => Some("ELEC"),
	49i16 => Some("GOR1"),
	50i16 => Some("GOR2"),
	51i16 => Some("GOR3"),
	52i16 => Some("GOR4"),
	53i16 => Some("GOR5"),
	54i16 => Some("TRE2"),
	55i16 => Some("SMBT"),
	56i16 => Some("SMGT"),
	57i16 => Some("SMRT"),
	58i16 => Some("SARG"),
	59i16 => Some("GOR2"),
	60i16 => Some("GOR4"),
	61i16 => Some("GOR3"),
	62i16 => Some("GOR5"),
	63i16 => Some("GOR1"),
	64i16 => Some("VILE"),
	65i16 => Some("CPOS"),
	66i16 => Some("SKEL"),
	67i16 => Some("FATT"),
	68i16 => Some("BSPI"),
	69i16 => Some("BOS2"),
	70i16 => Some("FCAN"),
	71i16 => Some("PAIN"),
	72i16 => Some("KEEN"),
	73i16 => Some("HDB1"),
	74i16 => Some("HDB2"),
	75i16 => Some("HDB3"),
	76i16 => Some("HDB4"),
	77i16 => Some("HDB5"),
	78i16 => Some("HDB6"),
	79i16 => Some("POB1"),
	80i16 => Some("POB2"),
	81i16 => Some("BRS1"),
	82i16 => Some("SGN2"),
	83i16 => Some("MEGA"),
	84i16 => Some("SSWV"),
	85i16 => Some("TLMP"),
	86i16 => Some("TLP2"),
	87i16 => None,
	88i16 => Some("BBRN"),
	89i16 => None,
	2001i16 => Some("SHOT"),
	2002i16 => Some("MGUN"),
	2003i16 => Some("LAUN"),
	2004i16 => Some("PLAS"),
	2005i16 => Some("CSAW"),
	2006i16 => Some("BFUG"),
	2007i16 => Some("CLIP"),
	2008i16 => Some("SHEL"),
	2010i16 => Some("ROCK"),
	2011i16 => Some("STIM"),
	2012i16 => Some("MEDI"),
	2013i16 => Some("SOUL"),
	2014i16 => Some("BON1"),
	2015i16 => Some("BON2"),
	2018i16 => Some("ARM1"),
	2019i16 => Some("ARM2"),
	2022i16 => Some("PINV"),
	2023i16 => Some("PSTR"),
	2024i16 => Some("PINS"),
	2025i16 => Some("SUIT"),
	2026i16 => Some("PMAP"),
	2028i16 => Some("COLU"),
	2035i16 => Some("BAR1"),
	2045i16 => Some("PVIS"),
	2046i16 => Some("BROK"),
	2047i16 => Some("CELL"),
	2048i16 => Some("AMMO"),
	2049i16 => Some("SBOX"),
	3001i16 => Some("TROO"),
	3002i16 => Some("SARG"),
	3003i16 => Some("BOSS"),
	3004i16 => Some("POSS"),
	3005i16 => Some("HEAD"),
	3006i16 => Some("SKUL"),
};

impl WadManager {
	pub fn bake_walls(&self) -> Result<(Vec<u64>, Vec<DoomPicture>, Vec<u64>, Vec<DoomPicture>, Vec<f32>), String> {
		let all_patchnames_raw = self.get_data("PNAMES")?;
		let patch_names: Vec<String> = all_patchnames_raw.get(4..)
			.ok_or_else(|| "Failed to get PNAMES data".to_string())?
			.chunks_exact(8)
			.map(|patchname| String::from_utf8_lossy(patchname)
                .trim_matches('\0')
                .to_uppercase()
			)
			.collect();

		let mut texture_lumps = Vec::new();
    	let texture1_raw = self.get_data("TEXTURE1")?;
    	let offsets1 = parse_texture_header(&texture1_raw, "TEXTURE1")?;
    	texture_lumps.push((texture1_raw, offsets1));

    	if let Ok(texture2_raw) = self.get_data("TEXTURE2") {
    	    let offsets2 = parse_texture_header(&texture2_raw, "TEXTURE2")?;
    	    texture_lumps.push((texture2_raw, offsets2));
    	}

		let sky_names = HashSet::from([
			pack_name_to_u64(b"RSKY1"),
			pack_name_to_u64(b"RSKY1"),
			pack_name_to_u64(b"RSKY2"),
			pack_name_to_u64(b"SKY1"),
			pack_name_to_u64(b"SKY2"),
			pack_name_to_u64(b"SKY3"),
			pack_name_to_u64(b"SKY4")
		]);

		let mut all_map_textures = Vec::new();
    	for (texture_raw, offsets) in &texture_lumps {
    	    for &offset in offsets {
    	        if let Ok(map_texture) = self.parse_texture_lump(texture_raw, offset as usize) {
    	            all_map_textures.push(map_texture);
    	        }
    	    }
    	}

		let cached_patches: Vec<Option<DoomPicture>> = patch_names
    		.par_iter() 
    		.map(|name| {
    		    self.get_data(name)
    		        .ok()
    		        .and_then(|data| decode_column_picture(&data).ok())
    		})
    		.collect();

		let baked_results: Vec<(u64, DoomPicture)> = all_map_textures
        	.into_par_iter()
        	.map_init(|| Vec::with_capacity(256 * 256),
				|thread_local_buffer, map_texture| {
    		    let width = map_texture.width as usize;
    		    let height = map_texture.height as usize;
    		    let total_pixels = width * height;
            	
            	thread_local_buffer.resize(total_pixels, 0xFF);
			
    		    for wad_patch in &map_texture.patches {
				    let patch_idx = wad_patch.patch as usize;
				    if patch_idx >= cached_patches.len() { continue; }

				    if let Some(ref patch_pic) = cached_patches[patch_idx] {
				        let p_width = patch_pic.width as usize;
                        let p_height = patch_pic.height as usize;
                    
                        let origin_x = wad_patch.originx as i32;
                        let origin_y = wad_patch.originy as i32;
                    
                        let start_x = if origin_x < 0 { (-origin_x) as usize } else { 0 };
                        let end_x = if origin_x + (p_width as i32) > width as i32 {
                            (width as i32 - origin_x) as usize
                        } else {
                            p_width
                        };
                    
                        let start_y = if origin_y < 0 { (-origin_y) as usize } else { 0 };
                        let end_y = if origin_y + (p_height as i32) > height as i32 {
                            (height as i32 - origin_y) as usize
                        } else {
                            p_height
                        };
                    
                        if start_x >= end_x || start_y >= end_y { continue; }
                    
                        for px in start_x..end_x {
                            let dest_x = (origin_x + px as i32) as usize;
                            let src_col_offset = px * p_height;
                        
                            for py in start_y..end_y {
                                let color_idx = patch_pic.raw_pixels[src_col_offset + py];
                            
                                if color_idx != 0xFF {
                                    let dest_y = (origin_y + py as i32) as usize;
                                    let dest_idx = dest_y * width + dest_x;
                                
                                    unsafe {
                                        *thread_local_buffer.get_unchecked_mut(dest_idx) = color_idx;
                                    }
                                }
                            }
                        }
				    }
				}

				let final_wall_pixels = thread_local_buffer.clone();
			
    		    let tex_name_packed = pack_name_to_u64(&map_texture.name);
    		    (tex_name_packed, DoomPicture {
    		        raw_pixels: final_wall_pixels,
    		        width: width as u32,
    		        height: height as u32,
    		        left_offset: 0,
    		        top_offset: 0,
    		    })
    		})
        .collect();

		let total_textures = texture_lumps.iter().map(|(_, offsets)| offsets.len()).sum();
	    let mut baked_textures = Vec::with_capacity(total_textures);
		let mut textures_names = Vec::with_capacity(total_textures);
		let mut baked_sky_textures = Vec::with_capacity(MAX_SKY);
		let mut sky_textures_names = Vec::with_capacity(MAX_SKY);
		let mut sky_widths = Vec::with_capacity(MAX_SKY);

		for (name, picture) in baked_results {
    	    if sky_names.contains(&name) {
    	        sky_textures_names.push(name);
    	        sky_widths.push(picture.width as f32);
    	        baked_sky_textures.push(picture); 
    	    } else {
				textures_names.push(name);
				baked_textures.push(picture);
			}
    	}
	    Ok((textures_names, baked_textures, sky_textures_names, baked_sky_textures, sky_widths))
	}

	pub fn parse_texture_lump(&self, lump_data: &[u8], offset: usize) -> Result<MapTexture, String> {
	    let data = &lump_data[offset..];

	    let name: [u8; 8] = data.get(0..8)
    	    .ok_or_else(|| "Failed to read texture name: lump data is too short".to_string())?
    	    .try_into()
    	    .map_err(|_| "Failed to parse texture name: invalid buffer size for [u8; 8]".to_string())?;

    	let masked = i32::from_le_bytes(data
    	    .get(8..12).ok_or_else(|| "Failed to read masked flag: lump data is too short".to_string())?
    	    .try_into()
    	    .map_err(|_| "Failed to parse masked flag: expected 4 bytes for i32".to_string())?) != 0;

    	let width = i16::from_le_bytes(data
    	    .get(12..14).ok_or_else(|| "Failed to read texture width: lump data is too short".to_string())?
    	    .try_into()
    	    .map_err(|_| "Failed to parse texture width: expected 2 bytes for i16".to_string())?);

    	let height = i16::from_le_bytes(data
    	    .get(14..16).ok_or_else(|| "Failed to read texture height: lump data is too short".to_string())?
    	    .try_into()
    	    .map_err(|_| "Failed to parse texture height: expected 2 bytes for i16".to_string())?);

    	// columndirectory, which is on 16..20, is obsolete

    	let patchcount = i16::from_le_bytes(data
    	    .get(20..22).ok_or_else(|| "Failed to read patch count: lump data is too short".to_string())?
    	    .try_into()
    	    .map_err(|_| "Failed to parse patch count: expected 2 bytes for i16".to_string())?);

    	let start_bytes: usize = 22;
    	let end_bytes = start_bytes + (patchcount as usize * 10);
		
    	let patches_bytes = data.get(start_bytes..end_bytes)
    	    .ok_or_else(|| format!(
    	        "Failed to read patch data: expected {} bytes for {} patches, but lump ended early", 
    	        patchcount * 10, patchcount
    	    ))?;

	    let wad_patches: Vec<MapPatch> = patches_bytes
			.chunks_exact(size_of::<MapPatch>())
			.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapPatch) }
    		})
    		.collect();

	    let patches: Vec<MapPatch> = wad_patches
	        .iter()
	        .map(|p| MapPatch {
	            originx: p.originx,
	            originy: p.originy,
	            patch: p.patch,
	            stepdir: p.stepdir,
	            colormap: p.colormap,
	        })
	        .collect();

	    Ok(MapTexture {
	        name,
	        masked,
	        width,
	        height,
	        patchcount,
	        patches,
	    })
	}

	pub fn bake_flats(&self) -> Result<(Vec<u64>, Vec<DoomPicture>), String> {
		let mut flats_map = HashMap::new();

		for wad in self.wads.iter() {
            let chunks = wad.data[wad.dir_offset..].chunks_exact(16);

			let f_start_pos = chunks.clone().enumerate().find_map(|(idx, chunk)| {
        	    let name = &chunk[8..16];
        	    if name.starts_with(b"F_START") || name.starts_with(b"FF_START") {
        	        Some(idx)
        	    } else { None }
        	});

        	let f_end_pos = chunks.clone().enumerate().find_map(|(idx, chunk)| {
        	    let name = &chunk[8..16];
        	    if name.starts_with(b"F_END") || name.starts_with(b"FF_END") {
        	        Some(idx)
        	    } else { None }
        	});

			if let (Some(start_idx), Some(end_idx)) = (f_start_pos, f_end_pos) {
                if start_idx >= end_idx { continue; }

                for idx in (start_idx + 1)..end_idx {
                    let chunk_pos = wad.dir_offset + idx * 16;
                    if chunk_pos + 16 > wad.data.len() { break; }

                    let chunk = &wad.data[chunk_pos..chunk_pos + 16];
                    let lump_offset = u32::from_le_bytes(chunk[0..4].try_into().unwrap()) as usize;
                    let lump_size = u32::from_le_bytes(chunk[4..8].try_into().unwrap()) as usize;
                    
                    if lump_size == 0 { continue; }

                    let name_bytes = &chunk[8..16];

                    let pic_bytes = &wad.data[lump_offset..lump_offset + lump_size];

					match decode_flat_picture(pic_bytes) {
                        Ok(picture) => { let _ = flats_map.insert(name_bytes, picture); },
						Err(err) => { eprintln!("{}: {}", String::from_utf8_lossy(name_bytes), err); }
                    }
                }
            }
		}
		if flats_map.is_empty() { return Err("No flats' pictures were baked!".to_string()); }

        let mut flats_names = Vec::with_capacity(flats_map.len());
        let mut baked_flats = Vec::with_capacity(flats_map.len());

        for (name_bytes, picture) in flats_map {
            flats_names.push(pack_name_to_u64(name_bytes));
            baked_flats.push(picture);
        }

	    Ok((flats_names, baked_flats))
	}

	pub fn bake_objects(&self) -> Result<(Vec<String>, Vec<DoomPicture>), String> {
		let mut objects_map = HashMap::new();

		for wad in self.wads.iter() {
            let chunks = wad.data[wad.dir_offset..].chunks_exact(16);

			let s_start_pos = chunks.clone().enumerate().find_map(|(idx, chunk)| {
        	    let name = &chunk[8..16];
        	    if name.starts_with(b"S_START") || name.starts_with(b"SS_START") {
        	        Some(idx)
        	    } else { None }
        	});

        	let s_end_pos = chunks.clone().enumerate().find_map(|(idx, chunk)| {
        	    let name = &chunk[8..16];
        	    if name.starts_with(b"S_END") || name.starts_with(b"SS_END") {
        	        Some(idx)
        	    } else { None }
        	});

			if let (Some(start_idx), Some(end_idx)) = (s_start_pos, s_end_pos) {
                if start_idx >= end_idx { continue; }

                for idx in (start_idx + 1)..end_idx {
                    let chunk_pos = wad.dir_offset + idx * 16;
                    if chunk_pos + 16 > wad.data.len() { break; }

                    let chunk = &wad.data[chunk_pos..chunk_pos + 16];
                    let lump_offset = u32::from_le_bytes(chunk[0..4].try_into().unwrap()) as usize;
                    let lump_size = u32::from_le_bytes(chunk[4..8].try_into().unwrap()) as usize;
                    
                    if lump_size == 0 { continue; }

                    let name_bytes = &chunk[8..16];
                    let name = String::from_utf8_lossy(name_bytes).trim_matches('\0').to_uppercase();

                    let pic_bytes = &wad.data[lump_offset..lump_offset + lump_size];

                    match decode_column_picture(pic_bytes) {
                        Ok(col_picture) => { 
                    	    let w = col_picture.width as usize;
                    	    let h = col_picture.height as usize;
                    	    let total_pixels = w * h;
							
                    	    let mut row_pixels = vec![0xFFu8; total_pixels];
							
                    	    for col in 0..w {
                    	        let src_col_offset = col * h;
                    	        for row in 0..h {
                    	            let color = col_picture.raw_pixels[src_col_offset + row];
                    	            row_pixels[row * w + col] = color;
                    	        }
                    	    }

                    	    let row_picture = DoomPicture {
                    	        raw_pixels: row_pixels,
                    	        width: col_picture.width,
                    	        height: col_picture.height,
                    	        left_offset: col_picture.left_offset,
                    	        top_offset: col_picture.top_offset,
                    	    };

                    	    let _ = objects_map.insert(name, row_picture); 
                    	},
						Err(err) => { eprintln!("{}: {}", name, err); }
                    }
                }
            }
		}
		if objects_map.is_empty() { return Err("No objects' pictures were baked!".to_string()); }

        let mut objects_names = Vec::with_capacity(objects_map.len());
        let mut baked_objects = Vec::with_capacity(objects_map.len());

        for (name, picture) in objects_map {
            objects_names.push(name);
            baked_objects.push(picture);
        }

	    Ok((objects_names, baked_objects))
	}
}

pub fn decode_column_picture(pic_data: &[u8]) -> Result<DoomPicture, String> {
    if pic_data.len() < 8 { 
        return Err("Picture data is too short".to_string()); 
    }

    let width = u16::from_le_bytes([pic_data[0], pic_data[1]]) as usize;
    let height = u16::from_le_bytes([pic_data[2], pic_data[3]]) as usize;
    let left_offset = i16::from_le_bytes([pic_data[4], pic_data[5]]);
    let top_offset = i16::from_le_bytes([pic_data[6], pic_data[7]]);

    let total_pixels = width * height;
    let mut sprite_pixels = vec![0xFFu8; total_pixels];
    
    let total_columns_size = width * 4;
    if pic_data.len() < 8 + total_columns_size {
        return Err("Picture data truncated in column directory".to_string());
    }

    let column_pointers = &pic_data[8..8 + total_columns_size];

    for col_idx in 0..width {
        let ptr_idx = col_idx * 4;
        let col_offset = u32::from_le_bytes([
            column_pointers[ptr_idx],
            column_pointers[ptr_idx+1],
            column_pointers[ptr_idx+2],
            column_pointers[ptr_idx+3]
        ]) as usize;

        let mut pointer = col_offset;

        loop {
            if pointer >= pic_data.len() { break; }
            let top_delta = pic_data[pointer];

            if top_delta == 0xFF {
                break; 
            }
        
            if pointer + 1 >= pic_data.len() { break; }
            let column_length = pic_data[pointer + 1] as usize;
            let pixel_data_start = pointer + 3;
        
            if pixel_data_start + column_length > pic_data.len() { break; }
            let pixels = &pic_data[pixel_data_start..pixel_data_start + column_length];
        
            let dest_col_offset = col_idx * height;

            for (i, &color_index) in pixels.iter().enumerate() {
                let row_idx = top_delta as usize + i;
                if row_idx < height {
                    let dest_index = dest_col_offset + row_idx;
                    unsafe {
                        *sprite_pixels.get_unchecked_mut(dest_index) = color_index;
                    }
                }
            }
        
            pointer += 4 + column_length;
        }
    }

    Ok(DoomPicture { 
        raw_pixels: sprite_pixels, 
        width: width as u32, 
        height: height as u32, 
        left_offset, 
        top_offset 
    })
}

pub fn decode_flat_picture(pic_data: &[u8]) -> Result<DoomPicture, String> {
    if pic_data.len() != 4096 { 
		Err("Flat picture data must be equal to 4096 bytes".to_string())
	} else {
		Ok(DoomPicture {
			raw_pixels: pic_data.to_vec(),
			width: 64,
			height: 64,
			left_offset: 0,
			top_offset: 0
		})
	}
}

fn parse_texture_header(raw_data: &[u8], lump_name: &str) -> Result<Vec<u32>, String> {
    let num_textures = u32::from_le_bytes(raw_data.get(0..4)
        .ok_or_else(|| format!("Failed to get num_textures from {}!", lump_name))?
        .try_into()
        .map_err(|_| format!("Failed to parse num_textures from {}; expected 4 bytes for u32", lump_name))?) as usize;
        
    let offsets_bytes = raw_data.get(4..4 + num_textures * 4)
        .ok_or_else(|| format!("Failed to get offsets_bytes from {}", lump_name))?;

    let offsets: Vec<u32> = offsets_bytes
        .chunks_exact(std::mem::size_of::<u32>())
        .map(|chunk| {
            unsafe { std::ptr::read_unaligned(chunk.as_ptr() as *const u32) }
        })
        .collect();

    Ok(offsets)
}

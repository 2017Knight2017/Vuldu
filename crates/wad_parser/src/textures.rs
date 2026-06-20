use crate::*;

#[repr(C, packed)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

#[derive(Debug, PartialEq)]
pub struct DoomPicture {
	pub raw_pixels: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub left_offset: i16,
	pub top_offset: i16,
} 

impl Wad {
	pub fn decode_column_picture(&self, sprite_lump: Lump) -> Option<DoomPicture> {
		let sprite_data = &self.data[sprite_lump.offset..sprite_lump.offset+sprite_lump.size];

		let width = u16::from_le_bytes(sprite_data.get(0..2)?.try_into().ok()?) as usize;
		let height = u16::from_le_bytes(sprite_data.get(2..4)?.try_into().ok()?) as usize;

		let left_offset = i16::from_le_bytes(sprite_data.get(4..6)?.try_into().ok()?);
		let top_offset = i16::from_le_bytes(sprite_data.get(6..8)?.try_into().ok()?);

		let mut sprite_pixels = vec![0xFFu8; width * height];

		let pixel_columns_chunks = sprite_data.get(8..8 + width*4)?.chunks_exact(4);

		for (col_idx, chunk) in pixel_columns_chunks.enumerate() {
		    let col_offset = u32::from_le_bytes(chunk.try_into().unwrap()) as usize;
		    let mut pointer = col_offset;

		    loop {
		        let top_delta = *sprite_data.get(pointer)?;
		        if top_delta == 0xFF {
		            break; 
		        }
			
		        let column_length = *sprite_data.get(pointer + 1)? as usize;
		        let pixel_data_start = pointer + 3;
			
		        let pixels = sprite_data.get(pixel_data_start..pixel_data_start + column_length)?;
			
		        for (i, &color_index) in pixels.iter().enumerate() {
		            let row_idx = top_delta as usize + i;
				
		            if row_idx < height {
		                let dest_index = row_idx * width + col_idx;
		                sprite_pixels[dest_index] = color_index;
		            }
		        }
			
		        pointer += 4 + column_length;
		    }
		}

		Some(DoomPicture { 
			raw_pixels: sprite_pixels, 
			width: width as u32, 
			height: height as u32, 
			left_offset, 
			top_offset 
		})
	}

	pub fn decode_flat_picture(&self, flat_lump: Lump) -> Option<DoomPicture> {
	    if flat_lump.size != 4096 { return None; }
	    let start = flat_lump.offset;
	    Some(DoomPicture {
			raw_pixels: self.data[start..start + 4096].to_vec(),
			width: 64,
			height: 64,
			left_offset: 0,
			top_offset: 0
		})
	}

	pub fn bake_walls(&self) -> Option<(Vec<String>, Vec<DoomPicture>)> {
		let all_patchnames_raw = self.get_data_by_lumpname("PNAMES")?;
		let patch_names: Vec<String> = all_patchnames_raw.get(4..)?
			.chunks_exact(8)
			.map(|patchname| String::from_utf8_lossy(patchname)
                .trim_matches('\0')
                .to_uppercase()
			)
			.collect();

		let texture1_raw = self.get_data_by_lumpname("TEXTURE1").expect("TEXTURE1 is not found!");
			
	    let num_textures = u32::from_le_bytes(texture1_raw.get(0..4)?.try_into().ok()?) as usize;
			
	    let offsets_bytes = texture1_raw.get(4..4 + num_textures * 4)?;
	    let offsets: &[u32] = bytemuck::cast_slice(offsets_bytes);

	    let mut baked_textures = Vec::with_capacity(num_textures);
		let mut textures_names = Vec::with_capacity(num_textures);

	    for &offset in offsets {
	        let map_texture = self.parse_texture_lump(&texture1_raw, offset as usize)?;

    		let name = String::from_utf8_lossy(&map_texture.name)
    		    .trim_matches('\0')
    		    .trim()
    		    .to_uppercase();
				
    		let width = map_texture.width as usize;
    		let height = map_texture.height as usize;

	        let mut final_wall_pixels = vec![0xFFu8; width * height * 4];

	        for wad_patch in &map_texture.patches {
	            let patch_idx = wad_patch.patch as usize;
			
	            if patch_idx >= patch_names.len() { continue; }
	            let patch_lump_name = &patch_names[patch_idx];

	            if let Some(patch_lump) = self.directory.get(patch_lump_name) {
	                if let Some(patch_pic) = self.decode_column_picture(*patch_lump) {
					
	                    for py in 0..patch_pic.height as usize {
	                        for px in 0..patch_pic.width as usize {
	                            let dest_x = (wad_patch.originx as usize).wrapping_add(px);
	                            let dest_y = (wad_patch.originy as usize).wrapping_add(py);

	                            if dest_x < width && dest_y < height {
	                                let src_idx = py * patch_pic.width as usize + px;
	                                let color_idx = patch_pic.raw_pixels[src_idx];

	                                if color_idx != 0xFF {
	                                    let dest_idx = dest_y * width + dest_x;
	                                    final_wall_pixels[dest_idx * 4 + 0] = color_idx;
										final_wall_pixels[dest_idx * 4 + 1] = color_idx;
										final_wall_pixels[dest_idx * 4 + 2] = color_idx;
										final_wall_pixels[dest_idx * 4 + 3] = 255;
	                                }
	                            }
	                        }
	                    }
	                }
	            }
	        }

			textures_names.push(name);

	        baked_textures.push(DoomPicture {
	            raw_pixels: final_wall_pixels,
	            width: width as u32,
	            height: height as u32,
	            left_offset: 0,
	            top_offset: 0,
	        });
	    }

	    Some((textures_names, baked_textures))
	}

	pub fn parse_texture_lump(&self, lump_data: &[u8], offset: usize) -> Option<MapTexture> {
	    let data = &lump_data[offset..];

	    let name: [u8; 8] = data.get(0..8)?.try_into().ok()?;
	    let masked = i32::from_le_bytes(data.get(8..12)?.try_into().ok()?) != 0;
	    let width = i16::from_le_bytes(data.get(12..14)?.try_into().ok()?);
	    let height = i16::from_le_bytes(data.get(14..16)?.try_into().ok()?);
	    // columndirectory, which is on 16..20, is obsolete
	    let patch_count = i16::from_le_bytes(data.get(20..22)?.try_into().ok()?);

	    let start_bytes: usize = 22;
	    let end_bytes = start_bytes + (patch_count as usize * 10);
	    let patches_bytes = data.get(start_bytes..end_bytes)?;

	    let wad_patches: &[MapPatch] = bytemuck::cast_slice(patches_bytes);

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

	    Some(MapTexture {
	        name,
	        masked,
	        width,
	        height,
	        patchcount: patch_count as i16,
	        patches,
	    })
	}

	pub fn bake_flats(&self) -> Option<(Vec<String>, Vec<DoomPicture>)> {
	    let mut flats_names = Vec::new();
	    let mut baked_flats = Vec::new();

	    for (name, &lump) in self.directory.iter() {
	        if let Some(flat_pic) = self.decode_flat_picture(lump) {
	            if name.starts_with("E") && name.contains("M") { continue; }
	            if name.starts_with("MAP") { continue; }

	            flats_names.push(name.clone());
	            baked_flats.push(flat_pic);
	        }
	    }

	    if baked_flats.is_empty() {
	        return None;
	    }

	    Some((flats_names, baked_flats))
	}
}
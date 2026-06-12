use crate::{Wad, Lump};

pub trait Sprite {
	fn into_raw_pixels(&self, sprite_lump: Lump) -> Option<DoomPicture>;
}

#[derive(Debug, PartialEq)]
pub struct DoomPicture {
	pub raw_pixels: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub left_offset: i16,
	pub top_offset: i16,
} 

impl Sprite for Wad {
	fn into_raw_pixels(&self, sprite_lump: Lump) -> Option<DoomPicture> {
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

		Some(DoomPicture { raw_pixels: sprite_pixels, width: width as u32, height: height as u32, left_offset, top_offset })
	}
}
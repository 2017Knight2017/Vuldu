use wad_parser::*;


#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::OnceLock;

	const WAD_PATH: &str = "../../assets/DOOM2.WAD";

	fn get_shared_wad() -> &'static Wad {
        static WAD_INSTANCE: OnceLock<Wad> = OnceLock::new();
        WAD_INSTANCE.get_or_init(|| {
            Wad::open(WAD_PATH).expect("Missing DOOM2.WAD in assets directory for integration tests")
        })
    }
	
	#[test]
	fn test_open() {
		let parsed_file = get_shared_wad();

		assert_eq!(parsed_file.directory.len(), 2608);
	}

	#[test]
	fn test_get_data_by_lumpname() {
		let parsed_file = get_shared_wad();
		
		let inexistent_lumpname = "A";
		assert!(parsed_file.get_data_by_lumpname(inexistent_lumpname).is_none());

		let valid_lumpname = "COLORMAP";
		assert!(parsed_file.get_data_by_lumpname(valid_lumpname).is_some());
	}

	#[test]
	fn test_into_raw_pixels() {
		let parsed_file = get_shared_wad();

		let valid_picture_lump = parsed_file.directory
			.get("HEADA1")
    		.expect("Lump HEADA1 not found in WAD directory");
		
		let picture = parsed_file.decode_column_picture(*valid_picture_lump);
		assert!(picture.is_some());

		let unwrapped_picture = picture.unwrap();
		assert_eq!(unwrapped_picture.width, 63);
		assert_eq!(unwrapped_picture.height, 66);
		assert_eq!(unwrapped_picture.left_offset, 31);
		assert_eq!(unwrapped_picture.top_offset, 67);
	}
}


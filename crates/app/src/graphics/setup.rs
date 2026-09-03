use std::{mem::offset_of, ops::Deref};

use bitflags::bitflags;
use engine::{STBarUi, UpdatableUiType, pack_sprite_u64};
use glam::Mat4;
use renderer::{
	ANIM_INFO_SIZE, AnimLevelInfo, LevelVertex, MAX_SKY, SafeRenderer, SpriteVertex,
	TextureDescriptor,
};
use rustc_hash::FxHashMap;
use wad_parser::{
	DoomPicture, GpuLevelVertex, GpuSpriteVertex, Level, NUM_UI, TextureId, WadManager,
	construct_map_name, to_u64,
};

bitflags! {
	pub struct GraphicsFlags: u32 {
		const NONE = 0;
		const WIREFRAME = 1 << 0;
		const BYTE_SHADOWS = 1 << 1;
		const FULL_BRIGHT = 1 << 2;
	}
}

impl GraphicsFlags {
	pub fn new(wireframe: bool, byte_shadows: bool) -> Self {
		let mut result = GraphicsFlags::NONE;

		if wireframe {
			result |= GraphicsFlags::WIREFRAME;
		}

		if byte_shadows {
			result |= GraphicsFlags::BYTE_SHADOWS;
		}

		result
	}
}

pub struct GraphicsContext {
	pub renderer: SafeRenderer,
	pub data: FxHashMap<u64, (TextureId, u32, u32, bool)>,
	pub ui_to_update: Vec<UpdatableUiType>,
	pub ui_db: Vec<Option<(TextureId, u32, u32)>>,
	pub cached_stbar_ui: STBarUi,
	pub offsets: Vec<(i16, i16)>,
	pub sector_heights: Vec<f32>,
	pub view_matrix: Mat4,
	pub flags: GraphicsFlags,
}

impl GraphicsContext {
	pub fn new(flags: GraphicsFlags) -> Self {
		Self {
			renderer: SafeRenderer::new(),
			data: FxHashMap::default(),
			ui_db: vec![None; NUM_UI],
			cached_stbar_ui: STBarUi::default(),
			ui_to_update: Vec::new(),
			offsets: Vec::new(),
			view_matrix: Mat4::default(),
			flags,
			sector_heights: Vec::new(),
		}
	}

	pub fn load_and_upload_textures(
		&mut self,
		wad_manager: &WadManager,
		map_num: u8,
	) -> Result<(), String> {
		let ((wall_names, wall_pics), (sky_names, sky_pics), sky_widths) = wad_manager
			.bake_walls(*MAX_SKY)
			.map_err(|e| format!("Wall baking failed: {e}"))?;
		println!("[load_and_upload_textures] walls are baked");

		let (flat_names, flat_pics) = wad_manager
			.bake_flats()
			.map_err(|e| format!("Flat baking failed: {e}"))?;
		println!("[load_and_upload_textures] flats are baked");

		let (obj_names, obj_pics) = wad_manager
			.bake_objects()
			.map_err(|e| format!("Object baking failed: {e}"))?;
		println!("[load_and_upload_textures] objects are baked");

		let (ui_shown, ui_pics) = wad_manager.bake_ui();
		println!("[load_and_upload_textures] ui is baked");

		let total_textures =
			wall_pics.len() + flat_pics.len() + obj_pics.len() + ui_pics.len() + *MAX_SKY;
		let total_pixels = 1 + sky_pics
			.iter()
			.chain(&obj_pics)
			.chain(&wall_pics)
			.chain(&flat_pics)
			.chain(&ui_pics)
			.map(|p| p.raw_pixels.len())
			.sum::<usize>();

		let mut all_pixels = Vec::with_capacity(total_pixels);
		let mut descriptors = Vec::with_capacity(total_textures);
		let mut current_gpu_id = 0;

		let mut sky_data: Vec<(&u64, DoomPicture, f32)> = sky_names
			.iter()
			.zip(sky_pics)
			.zip(sky_widths)
			.map(|((n, p), w)| (n, p, w))
			.collect();
		sky_data.sort_by_key(|trio| trio.0);
		current_gpu_id += *MAX_SKY as u32;

		let mut sky_widths_no_name = Vec::with_capacity(sky_data.len());
		for (_, pic, width) in sky_data {
			descriptors.push(TextureDescriptor {
				width: pic.width,
				height: pic.height,
				pixel_offset: all_pixels.len(),
			});
			all_pixels.extend_from_slice(&pic.raw_pixels);
			sky_widths_no_name.push(width);
		}

		let padding_needed = MAX_SKY.deref().saturating_sub(descriptors.len());
		for _ in 0..padding_needed {
			descriptors.push(TextureDescriptor {
				width: 1,
				height: 1,
				pixel_offset: all_pixels.len(),
			});
		}
		all_pixels.push(0);

		self.offsets.reserve(obj_pics.len());
		for (idx, pic) in obj_pics.iter().enumerate() {
			let name = obj_names[idx];
			if name.iter().all(|&char| char == b'\0') {
				continue;
			}

			self.offsets.push((pic.left_offset, pic.top_offset));
			register_sprite(
				&mut self.data,
				name,
				(TextureId(current_gpu_id), pic.width, pic.height),
			);

			descriptors.push(TextureDescriptor {
				width: pic.width,
				height: pic.height,
				pixel_offset: all_pixels.len(),
			});
			all_pixels.extend_from_slice(&pic.raw_pixels);
			current_gpu_id += 1;
		}

		let mut anim_map: FxHashMap<u64, u32> = FxHashMap::from_iter([
			(to_u64(b"FWATER1"), 4),
			(to_u64(b"SWATER1"), 4),
			(to_u64(b"LAVA1"), 4),
			(to_u64(b"RROCK05"), 4),
			(to_u64(b"SLIME01"), 4),
			(to_u64(b"SLIME05"), 4),
			(to_u64(b"SLIME09"), 4),
			(to_u64(b"BLODGR1"), 4),
			(to_u64(b"BLODRIP1"), 4),
			(to_u64(b"BFALL1"), 4),
			(to_u64(b"SFALL1"), 4),
			(to_u64(b"WFALL1"), 4),
			(to_u64(b"DBRAIN1"), 4),
			(to_u64(b"NUKAGE1"), 3),
			(to_u64(b"SLADRIP1"), 3),
			(to_u64(b"GSTFONT1"), 3),
			(to_u64(b"FIRELAV2"), 3),
			(to_u64(b"FIREMAG1"), 3),
			(to_u64(b"ROCKRED1"), 3),
			(to_u64(b"FIREWALA"), 3),
			(to_u64(b"BLOOD1"), 3),
			(to_u64(b"FIREBLU1"), 2),
		]);

		let mut anim_level_info = Vec::new();
		anim_level_info.resize_with(*ANIM_INFO_SIZE, || AnimLevelInfo {
			texture: 0,
			frames: 0,
			_padding: [0, 0],
		});

		for (tex_names, pics) in [(wall_names, wall_pics), (flat_names, flat_pics)] {
			for (idx, pic) in pics.iter().enumerate() {
				let name = tex_names[idx];

				if let Some(frames) = anim_map.remove(&name) {
					let gpu_idx = current_gpu_id as usize;
					let frames_len = frames as usize;
					if gpu_idx + frames_len - 1 < *ANIM_INFO_SIZE {
						for i in 0..frames_len {
							anim_level_info[gpu_idx + i].texture = current_gpu_id;
							anim_level_info[gpu_idx + i].frames = frames;
						}
					}
				}

				self.data.insert(
					name,
					(TextureId(current_gpu_id), pic.width, pic.height, false),
				);
				descriptors.push(TextureDescriptor {
					width: pic.width,
					height: pic.height,
					pixel_offset: all_pixels.len(),
				});
				all_pixels.extend_from_slice(&pic.raw_pixels);

				current_gpu_id += 1;
			}
		}

		for (pic, ui_insert_idx) in ui_pics.into_iter().zip(ui_shown.ones()) {
			descriptors.push(TextureDescriptor {
				width: pic.width,
				height: pic.height,
				pixel_offset: all_pixels.len(),
			});
			all_pixels.extend_from_slice(&pic.raw_pixels);

			let tex_id = TextureId(current_gpu_id);
			self.ui_db[ui_insert_idx] = Some((tex_id, pic.width, pic.height));

			current_gpu_id += 1;
		}

		self.renderer
			.pin()
			.uploadTextureArray(&descriptors, &all_pixels, &sky_widths_no_name);
		self.renderer.pin().uploadAnimLevelInfo(&anim_level_info);

		let map_name = construct_map_name(wad_manager.is_doom1, map_num);
		let palettes = wad_manager
			.get_palettes(&map_name)
			.map_err(|e| format!("PLAYPAL upload failed: {e}"))?;
		self.renderer.pin().uploadPalettes(&palettes);

		let colormap = wad_manager
			.get_colormap(&map_name)
			.map_err(|e| format!("COLORMAP upload failed: {e}"))?;
		self.renderer.pin().uploadColormap(colormap);

		Ok(())
	}

	pub fn setup_level_geometry(&mut self, level: &mut Level) {
		println!("Building map geometry...");
		let (wall_vertices, wall_indices) = level.get_walls_vertices(&self.data);
		println!("Walls geometry has been built");
		let (flat_vertices, flat_indices) = level.get_flats_vertices(&self.data);
		println!("Flats geometry has been built");
		let (obj_gpu_vertices, obj_indices) = level.get_objects_vertices();
		let (ui_gpu_vertices, ui_indices) = level.get_ui_vertices();

		self.sector_heights = level
			.state
			.sectors
			.iter()
			.flat_map(|sec| [sec.floor_h, sec.ceil_h])
			.collect();

		let mut level_vertices: Vec<LevelVertex> =
			wall_vertices.into_iter().map(to_level_vertex).collect();
		let mut level_indices = wall_indices;

		let vertex_offset = level_vertices.len() as u32;
		level_vertices.extend(flat_vertices.into_iter().map(to_level_vertex));
		for idx in flat_indices {
			level_indices.push(vertex_offset + idx);
		}

		let obj_vertices: Vec<SpriteVertex> =
			obj_gpu_vertices.into_iter().map(to_sprite_vertex).collect();
		let ui_vertices: Vec<SpriteVertex> =
			ui_gpu_vertices.into_iter().map(to_sprite_vertex).collect();

		self.renderer.pin().setFlags(self.flags.bits());
		self.renderer
			.pin()
			.updateLevelGeometry(&level_vertices, &level_indices);
		self.renderer
			.pin()
			.updateObjectGeometry(&obj_vertices, &obj_indices);
		self.renderer
			.pin()
			.updateUiGeometry(&ui_vertices, &ui_indices);
		self.renderer.pin().initSectorHeights(&self.sector_heights);
	}
}

fn register_sprite(
	texture_data_map: &mut FxHashMap<u64, (TextureId, u32, u32, bool)>,
	lump_name: &[u8],
	texture_tuple: (TextureId, u32, u32),
) {
	let (id, w, h) = texture_tuple;

	let last_non_zero = lump_name.iter().rposition(|&b| b != b'\0').unwrap();
	let normed_name = &lump_name[..=last_non_zero];

	let prefix = &normed_name[..4];
	let frame1 = normed_name[4] as char;
	let view1 = normed_name[5] - b'0';

	let key1 = pack_sprite_u64(prefix, frame1, view1);
	texture_data_map.insert(key1, (id, w, h, false));

	if normed_name.len() == 8 {
		let frame2 = normed_name[6] as char;
		let view2 = normed_name[7] - b'0';

		let key2 = pack_sprite_u64(prefix, frame2, view2);
		texture_data_map.insert(key2, (id, w, h, true));
	}
}

fn to_level_vertex(vertex: GpuLevelVertex) -> LevelVertex {
	assert_eq!(size_of::<GpuLevelVertex>(), size_of::<LevelVertex>());

	assert_eq!(
		offset_of!(GpuLevelVertex, pos),
		offset_of!(LevelVertex, pos)
	);
	assert_eq!(
		offset_of!(GpuLevelVertex, texture_pos),
		offset_of!(LevelVertex, texture_pos)
	);
	assert_eq!(
		offset_of!(GpuLevelVertex, light_level),
		offset_of!(LevelVertex, light_level)
	);
	assert_eq!(
		offset_of!(GpuLevelVertex, texture_id),
		offset_of!(LevelVertex, texture_id)
	);
	assert_eq!(
		offset_of!(GpuLevelVertex, floor_tex_id),
		offset_of!(LevelVertex, floor_tex_id)
	);
	assert_eq!(
		offset_of!(GpuLevelVertex, scroll_dir),
		offset_of!(LevelVertex, scroll_dir)
	);

	LevelVertex {
		pos: vertex.pos,
		texture_pos: vertex.texture_pos,
		light_level: vertex.light_level,
		texture_id: vertex.texture_id,
		floor_tex_id: vertex.floor_tex_id,
		scroll_dir: vertex.scroll_dir,
		plane_a: vertex.plane_a,
		plane_b: vertex.plane_b,
		inv_tex_h: vertex.inv_tex_h,
	}
}

fn to_sprite_vertex(vertex: GpuSpriteVertex) -> SpriteVertex {
	assert_eq!(size_of::<GpuSpriteVertex>(), size_of::<SpriteVertex>());

	assert_eq!(
		offset_of!(GpuSpriteVertex, pos),
		offset_of!(SpriteVertex, pos)
	);
	assert_eq!(
		offset_of!(GpuSpriteVertex, texture_pos),
		offset_of!(SpriteVertex, texture_pos)
	);

	SpriteVertex {
		pos: vertex.pos,
		texture_pos: vertex.texture_pos,
	}
}

use hecs::World;
use renderer::{MAX_SKY, ObjectInstance, SafeRenderer, TextureDescriptor, UiInstance, UniformBufferObject, Vertex};
use engine::{CurrentSector, EYEHEIGHT, Health, MonsterRotation, PlayerInfo, PlayerMarker, PlayerRotation, Position, SpriteAnimation, get_stbar, pack_sprite_u64, point_to_angle};
use glam::{Mat4, Vec3};
use micropool::iter::*;
use rustc_hash::FxHashMap;
use wad_parser::{DoomPicture, GpuVertex, Level, LineId, NUM_UI, SectorState, TextureId, Ui, WadManager, construct_map_name};
use winit::window::Window;
use std::f64::consts::TAU;

const FOV_ANGLE: f32 = 90.0;

pub struct GraphicsContext {
    pub renderer: Option<SafeRenderer>,
    pub data: FxHashMap<u64, (TextureId, u32, u32, bool)>,
    pub ui_db: [Option<(TextureId, u32, u32)>; NUM_UI],
    pub cached_ui_instances: Vec<UiInstance>,
    pub offsets: Vec<(i16, i16)>,
    pub view_matrix: Mat4,
}

impl GraphicsContext {
	pub fn new() -> Self {
		Self { 
			renderer: None, 
            data: FxHashMap::default(), 
            ui_db: [None; NUM_UI],
            cached_ui_instances: Vec::new(),
            offsets: Vec::new(), 
            view_matrix: Mat4::default()
		}
	}

	pub fn load_and_upload_textures(&mut self, renderer: &mut SafeRenderer, wad_manager: &WadManager, map_num: u8) -> Result<(), String> {
        let max_sky = *MAX_SKY.get().unwrap();

        let (wall_names, wall_pics, sky_names, sky_pics, sky_widths) = 
            wad_manager.bake_walls(max_sky).map_err(|e| format!("Wall baking failed: {e}"))?;
        println!("[load_and_upload_textures] walls are baked");

        let (flat_names, flat_pics) = 
            wad_manager.bake_flats().map_err(|e| format!("Flat baking failed: {e}"))?;
        println!("[load_and_upload_textures] flats are baked");

        let (obj_names, obj_pics) = 
            wad_manager.bake_objects().map_err(|e| format!("Object baking failed: {e}"))?;
        println!("[load_and_upload_textures] objects are baked");

        let (ui_shown, ui_pics) = wad_manager.bake_ui();
        println!("[load_and_upload_textures] ui is baked");

        let total_textures = wall_pics.len() + flat_pics.len() + obj_pics.len() + ui_pics.len() + max_sky;
        let total_pixels = 1 + sky_pics.iter()
            .chain(&obj_pics)
            .chain(&wall_pics)
            .chain(&flat_pics)
            .chain(&ui_pics)
            .map(|p| p.raw_pixels.len())
            .sum::<usize>();

        let mut all_pixels = Vec::with_capacity(total_pixels);
        let mut descriptors = Vec::with_capacity(total_textures);
        let mut current_gpu_id = 0;

        let mut sky_data: Vec<(&u64, DoomPicture, f32)> = sky_names.iter()
            .zip(sky_pics)
            .zip(sky_widths)
            .map(|((n, p), w)| (n, p, w))
            .collect();
        sky_data.sort_by_key(|trio| trio.0);
        current_gpu_id += max_sky as u32;

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

        let padding_needed = max_sky.saturating_sub(descriptors.len());                    
        for _ in 0..padding_needed {
            descriptors.push(TextureDescriptor {
                width: 1, height: 1, pixel_offset: all_pixels.len(),
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
            register_sprite(&mut self.data, name, (TextureId(current_gpu_id), pic.width, pic.height));
            
            descriptors.push(TextureDescriptor {
                width: pic.width, height: pic.height, pixel_offset: all_pixels.len(),
            });
            all_pixels.extend_from_slice(&pic.raw_pixels);
            current_gpu_id += 1;
        }

        for (tex_names, pics) in [(wall_names, wall_pics), (flat_names, flat_pics)] {
            for (idx, pic) in pics.iter().enumerate() {
                let name = tex_names[idx];
                self.data.insert(name, (TextureId(current_gpu_id), pic.width, pic.height, false));
                descriptors.push(TextureDescriptor {
                    width: pic.width, height: pic.height, pixel_offset: all_pixels.len(),
                });
                all_pixels.extend_from_slice(&pic.raw_pixels);
                current_gpu_id += 1;
            }
        }

        let mut ui_insert_idx = 0;
        for pic in ui_pics {
            descriptors.push(TextureDescriptor {
                width: pic.width, 
                height: pic.height, 
                pixel_offset: all_pixels.len(),
            });
            all_pixels.extend_from_slice(&pic.raw_pixels);

            while !ui_shown[ui_insert_idx] { ui_insert_idx += 1; }
            let tex_id = TextureId(current_gpu_id);
            self.ui_db[ui_insert_idx] = Some((tex_id, pic.width, pic.height));

            ui_insert_idx += 1;
            current_gpu_id += 1;
        }

        renderer.upload_texture_array(&descriptors, &all_pixels, &sky_widths_no_name);

        let map_name = construct_map_name(wad_manager.is_doom1, map_num);
        let palettes = wad_manager.get_palettes(&map_name).map_err(|e| format!("PLAYPAL upload failed: {e}"))?;
        renderer.upload_palettes(&palettes);

        let colormap = wad_manager.get_colormap(&map_name).map_err(|e| format!("COLORMAP upload failed: {e}"))?;
        renderer.upload_colormap(colormap);

        Ok(())
    }

    pub fn setup_level_geometry(&mut self, renderer: &mut SafeRenderer, level: &Level) -> Vec<Vec<LineId>> {
        println!("Building map geometry...");
        let (wall_vertices, wall_indices) = level.get_walls_vertices(&self.data);
        println!("Walls geometry has been built");
        let (flat_vertices, flat_indices, sector_lines) = level.get_flats_vertices(&self.data);
        println!("Flats geometry has been built");
        let (obj_gpu_vertices, obj_indices) = level.get_objects_vertices();
        let (ui_gpu_vertices, ui_indices) = level.get_ui_vertices();

        let mut level_vertices: Vec<Vertex> = wall_vertices.into_iter().map(|v| vertex_to_vertex(v)).collect();
        let mut level_indices = wall_indices;

        let vertex_offset = level_vertices.len() as u32; 
        level_vertices.extend(flat_vertices.into_iter().map(|v| vertex_to_vertex(v)));
        for idx in flat_indices {
            level_indices.push(vertex_offset + idx);
        }

        let obj_vertices: Vec<Vertex> = obj_gpu_vertices.into_iter().map(|v| vertex_to_vertex(v)).collect();
        let ui_vertices: Vec<Vertex> = ui_gpu_vertices.into_iter().map(|v| vertex_to_vertex(v)).collect();

        renderer.update_object_instances(&[]);
        renderer.update_ui_instances(&[]);
        renderer.update_level_geometry(&level_vertices, &level_indices);
        renderer.update_object_geometry(&obj_vertices, &obj_indices);
        renderer.update_ui_geometry(&ui_vertices, &ui_indices);

        sector_lines
    }

    pub fn render(&mut self, window: &Window, world: &World, level: &Level, player_info: &PlayerInfo, alpha: f32) {
        let obj_instances = self.collect_object_instances(world, &level.state.sectors, alpha);

        if self.cached_ui_instances.is_empty() { 
            let mut health_query = world.query_one::<&Health>(player_info.entity);
            self.cached_ui_instances = self.collect_ui_instances(player_info, health_query.get().unwrap()); 
        }
        
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let aspect_ratio = size.width as f32 / size.height as f32;
        let proj = Mat4::perspective_rh(FOV_ANGLE.to_radians(), aspect_ratio, 1.0, 10000.0);

        update_camera_from_player(&mut self.view_matrix, world, alpha);

        let ubo = UniformBufferObject {
            model: Mat4::IDENTITY.to_cols_array(),
            view: self.view_matrix.to_cols_array(),
            proj: proj.to_cols_array(),
        };

        if let Some(renderer) = &mut self.renderer {
            renderer.update_ui_instances(&self.cached_ui_instances);
            renderer.update_object_instances(&obj_instances);
            renderer.start_frame(&ubo);
            renderer.draw_level();
            renderer.draw_objects();
            renderer.draw_ui();
            renderer.end_frame();
        }
    }

	fn collect_object_instances(&self, world: &World, sectors: &[SectorState], alpha: f32) -> Vec<ObjectInstance> {
	    let mut player_pos = Vec3::ZERO;
	
	    for pos in world.query::<&Position>().with::<&PlayerMarker>().iter() {
	        let lerped_x = pos.prev_x * (1.0 - alpha) + pos.x * alpha;
	        let lerped_y = pos.prev_y * (1.0 - alpha) + pos.y * alpha;
	        let lerped_z = pos.prev_z * (1.0 - alpha) + pos.z * alpha;
	        player_pos = Vec3::new(lerped_x, lerped_y, lerped_z);
	    }

		let mut entities_query = world.query::<(&Position, &MonsterRotation, &CurrentSector, &SpriteAnimation)>();
		let entities_to_process = entities_query
			.iter()
			.collect::<Vec<(&Position, &MonsterRotation, &CurrentSector, &SpriteAnimation)>>();

		let sprite_offsets = &self.offsets;

		let nested_instances = entities_to_process 
        	.par_iter()
        	.with_thread_pool(micropool::split_by_threads())
        	.map(|(pos, rot, current_sector, anim)| {
				let lerped_x = pos.prev_x * (1.0 - alpha) + pos.x * alpha;
	    	    let lerped_y = pos.prev_y * (1.0 - alpha) + pos.y * alpha;
	    	    let lerped_z = pos.prev_z * (1.0 - alpha) + pos.z * alpha;	
			
	    	    let monster_pos = Vec3::new(lerped_x, lerped_y, lerped_z);

	    	    let monster_angle = match rot.move_dir {
					Some(dir) => (dir as u32) << 29,
					None => 0
				};

	    	    let to_player = player_pos - monster_pos;
				let angle_to_player = point_to_angle(to_player.x, to_player.z);

				let view_angle = angle_to_player.wrapping_sub(monster_angle);

				let sector_offset = 0x10000000;
				let shifted_angle = view_angle.wrapping_add(sector_offset);

				let sprite_rotation = ((shifted_angle >> 29) + 1) as u8;
			
	    	    let cached = anim.cached_rotations[sprite_rotation as usize];
			
        		let tex_id = cached.tex_id;
        		let tex_width = cached.width;
        		let tex_height = cached.height;
        		let need_flip = cached.need_flip;
				// first 16 indices are reserved for sky textures,
				// so we have to subtract MAX_SKY from the actual index
				let (left_offset, top_offset) = sprite_offsets[tex_id.0 as usize - MAX_SKY.get().unwrap()];  

				let mut final_width = tex_width as f32;
        		let mut final_left_offset = left_offset as f32;

				if need_flip {
        		    final_width = -final_width;
        		    final_left_offset = tex_width as f32 - final_left_offset;
        		}

				let sector = &sectors[current_sector.0.0];

				let clamped_light = sector.light.clamp(0, 255) as f32;
	    	    let modern_light = clamped_light / 255.0;
	    	    let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	    	    ObjectInstance {
	    	        pos: [lerped_x, lerped_y, lerped_z],
	    	        sprite_offset: [final_left_offset, (top_offset + anim.top_offset_shift) as f32],
					sprite_size: [final_width, tex_height as f32],
	    	        light_level: modern_light,
	    	        texture_id: tex_id.0,
	    	        colormap_idx,
	    	    }
	    	})
			.collect_per_thread::<Vec<ObjectInstance>>();

		let total_count: usize = nested_instances.iter().map(|v| v.len()).sum();

		let mut instances = Vec::with_capacity(total_count);

		for mut thread_vec in nested_instances {
		    instances.append(&mut thread_vec); 
		}

	    instances
	}

    fn collect_ui_instances(&self, player_info: &PlayerInfo, player_health: &Health) -> Vec<UiInstance> {
        // TODO: make get_stbar run only when any of PlayerInfo params is changed
        get_stbar(
            player_info, 
            player_health,
            self.ui_db[Ui::STBAR as usize].unwrap().2 as f32, 
            (self.ui_db[Ui::STTNUM0 as usize].unwrap().1 as f32,
            self.ui_db[Ui::STYSNUM0 as usize].unwrap().1 as f32)
        )
            .iter()
            .map(|(ui, x, y)| {
                let (tex_id, width, height) = self.ui_db[*ui as usize].unwrap();
                UiInstance {
                    pos: [*x, *y],
                    sprite_size: [width as f32, height as f32],
                    texture_id: tex_id.0
                }
            })
            .collect()
    }
}

fn register_sprite(
    texture_data_map: &mut FxHashMap<u64, (TextureId, u32, u32, bool)>, 
    lump_name: &[u8], 
    texture_tuple: (TextureId, u32, u32)
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

fn vertex_to_vertex(vertex: GpuVertex) -> Vertex {
    Vertex { 
        pos: vertex.pos, 
        texture_pos: vertex.texture_pos, 
        light_level: vertex.light_level, 
        texture_id: vertex.texture_id, 
        colormap_idx: vertex.colormap_idx, 
        floor_tex_id: vertex.floor_tex_id 
    }
}

fn update_camera_from_player(view_matrix: &mut Mat4, world: &World, alpha: f32) {
    for (position, rotation, _player) in world.query::<(&Position, &PlayerRotation, &PlayerMarker)>().iter() {

        let prev_pos = glam::vec3(position.prev_x, position.prev_y + EYEHEIGHT, position.prev_z);
        let current_pos = glam::vec3(position.x, position.y + EYEHEIGHT, position.z);
        let interpolated_pos = prev_pos + (current_pos - prev_pos) * alpha;

        let angle_diff = rotation.angle.wrapping_sub(rotation.prev_angle) as i32;
        let interpolated_diff = (angle_diff as f64 * alpha as f64) as i32;
        let interpolated_angle_u32 = rotation.prev_angle.wrapping_add_signed(interpolated_diff);

        let angle_normalized = interpolated_angle_u32 as f64 / u32::MAX as f64;
        let angle_rad = (angle_normalized * TAU) as f32;

        let target_dir = glam::vec3(f32::sin(angle_rad), 0.0, f32::cos(angle_rad));
        let camera_target = interpolated_pos + target_dir;

        let camera_up = glam::vec3(0.0, 1.0, 0.0);

        *view_matrix = glam::Mat4::look_at_rh(interpolated_pos, camera_target, camera_up);
    }
}

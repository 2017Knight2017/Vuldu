use crate::{App, MAX_SKY};
use renderer::ObjectInstance;
use engine::{CurrentSector, PlayerMarker, Position, MonsterRotation, SpriteAnimation, point_to_angle};
use glam::Vec3;
use micropool::iter::*;

impl App {
	pub fn collect_object_instances(&self, alpha: f32) -> Vec<ObjectInstance> {
	    let mut player_pos = Vec3::ZERO;
	
	    for (position, _player_marker) in self.world.query::<(&Position, &PlayerMarker)>().iter() {
	        let lerped_x = position.prev_x * (1.0 - alpha) + position.x * alpha;
	        let lerped_y = position.prev_y * (1.0 - alpha) + position.y * alpha;
	        let lerped_z = position.prev_z * (1.0 - alpha) + position.z * alpha;
	        player_pos = Vec3::new(lerped_x, lerped_y, lerped_z);
	    }

		let mut entities_query = self.world.query::<(&Position, &MonsterRotation, &CurrentSector, &SpriteAnimation)>();
		let entities_to_process: Vec<(&Position, &MonsterRotation, &CurrentSector, &SpriteAnimation)> = entities_query
			.iter()
			.collect();

		let sprite_offsets = &self.sprite_offsets;
    	let sectors = &self.map.sectors;

		let nested_instances = entities_to_process 
        	.par_iter()
        	.with_thread_pool(micropool::split_by_threads())
        	.map(|(position, rotation, sector_idx, anim)| {
				let lerped_x = position.prev_x * (1.0 - alpha) + position.x * alpha;
	    	    let lerped_y = position.prev_y * (1.0 - alpha) + position.y * alpha;
	    	    let lerped_z = position.prev_z * (1.0 - alpha) + position.z * alpha;	
			
	    	    let monster_pos = Vec3::new(lerped_x, lerped_y, lerped_z);

	    	    let monster_angle = rotation.move_dir << 29;

	    	    let to_player = player_pos - monster_pos;
				let angle_to_player = point_to_angle(-to_player.x, to_player.z);

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
				let (left_offset, top_offset) = sprite_offsets[tex_id as usize - MAX_SKY];  

				let mut final_width = tex_width as f32;
        		let mut final_left_offset = left_offset as f32;

				if need_flip {
        		    final_width = -final_width;
        		    final_left_offset = tex_width as f32 - final_left_offset;
        		}

				let sector = sectors[sector_idx.0];

				let clamped_light = sector.lightlevel.clamp(0, 255) as f32;
	    	    let modern_light = clamped_light / 255.0;
	    	    let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	    	    ObjectInstance {
	    	        pos: [lerped_x, lerped_y, lerped_z],
	    	        sprite_offset: [final_left_offset, (top_offset + anim.top_offset_shift) as f32],
					sprite_size: [final_width, tex_height as f32],
	    	        light_level: modern_light,
	    	        texture_id: tex_id,
	    	        colormap_idx: colormap_idx,
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
}

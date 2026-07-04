use crate::App;
use renderer::ObjectInstance;
use engine::{DB, Health, PlayerMarker, SpriteAnimation, Transform};
use glam::{Vec3};
use std::f64::consts::TAU;
use hecs::Entity;

impl App {
	pub fn collect_object_instances(&self, alpha: f32) -> Vec<ObjectInstance> {
	    let mut instances = Vec::new();

	    let mut player_pos = Vec3::ZERO;
	
	    for (transform, _player_marker) in self.world.query::<(&Transform, &PlayerMarker)>().iter() {
	        let lerped_x = transform.prev_x * (1.0 - alpha) + transform.x * alpha;
	        let lerped_y = transform.prev_y * (1.0 - alpha) + transform.y * alpha;
	        let lerped_z = transform.prev_z * (1.0 - alpha) + transform.z * alpha;
	        player_pos = Vec3::new(lerped_x, lerped_y, lerped_z);
	    }

	    for (entity, transform, anim) in self.world.query::<(Entity, &Transform, &SpriteAnimation)>().iter() {   
			let current_state = match anim.current_state {
				Some(state) => state,
				None => continue
			};

	        let lerped_x = transform.prev_x * (1.0 - alpha) + transform.x * alpha;
	        let mut lerped_y = transform.prev_y * (1.0 - alpha) + transform.y * alpha;
	        let lerped_z = transform.prev_z * (1.0 - alpha) + transform.z * alpha;

			if self.world.get::<&Health>(entity).is_ok() {
				lerped_y += 5.0;
			}
		
	        let monster_pos = Vec3::new(lerped_x, lerped_y, lerped_z);

	        let monster_angle = lerp_angle(transform.prev_angle, transform.angle, alpha);

	        let to_player = player_pos - monster_pos;
			let mut rad_to_player = f64::atan2(to_player.z.into(), (-to_player.x).into());
			if rad_to_player < 0.0 {
			    rad_to_player += TAU;
			}

			let angle_to_player = ((rad_to_player / TAU) * u32::MAX as f64) as u32;

			let view_angle = angle_to_player.wrapping_sub(monster_angle);

			let sector_offset = 0x10000000;
			let shifted_angle = view_angle.wrapping_add(sector_offset);

			let sprite_rotation = (shifted_angle >> 29) + 1;  // same as shifted_angle / 0x20000000

			let db = DB.get().expect("DB is not initialized!");
			let current_state_data = db.states[&current_state];
		
	        let tex_prefix = current_state_data.sprite;
	        let frame_letter = (b'A' + current_state_data.frame as u8) as char;
		
	        let mut final_tex_name = format!("{}{}{}", tex_prefix, frame_letter, sprite_rotation);
			let mut need_flip = false;

	        if !self.texture_data.contains_key(&final_tex_name) {
				let flipped_name = format!("{}_FLIP", final_tex_name);
	            if self.texture_data.contains_key(&flipped_name) {
            	    final_tex_name = flipped_name;
					need_flip = true;
            	} else {
            	    final_tex_name = format!("{}{}0", tex_prefix, frame_letter);
            	}
			}

	        let (tex_id, tex_width, tex_height) = self.texture_data[&final_tex_name];
			let (left_offset, top_offset) = self.sprite_offsets[tex_id as usize];

			let mut final_width = tex_width as f32;
        	let mut final_left_offset = left_offset as f32;

        	if need_flip {
        	    final_width = -final_width;
        	    final_left_offset = tex_width as f32 - final_left_offset;
        	}

			let sector_idx = self.map.get_sector_by_pos(lerped_x, lerped_z);
			let sector = self.map.sectors[sector_idx];

			let clamped_light = sector.lightlevel.clamp(0, 255) as f32;
	        let modern_light = clamped_light / 255.0;
	        let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	        instances.push(ObjectInstance {
	            pos: [lerped_x, lerped_y, lerped_z],
	            sprite_offset: [final_left_offset, top_offset as f32],
				sprite_size: [final_width, tex_height as f32],
	            light_level: modern_light,
	            texture_id: tex_id,
	            colormap_idx: colormap_idx,
	        });
	    }

	    instances
	}
}


fn lerp_angle(from: u32, to: u32, alpha: f32) -> u32 {
    let diff = (to as i32).wrapping_sub(from as i32);
    
    let lerped_diff = (diff as f64 * alpha as f64) as i32;
    
    from.wrapping_add_signed(lerped_diff)
}

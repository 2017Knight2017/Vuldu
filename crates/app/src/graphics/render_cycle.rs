use std::f64::consts::TAU;

use engine::*;
use glam::{Mat4, Vec3};
use hecs::{Entity, World};
use micropool::iter::*;
use renderer::{MAX_SKY, MVP, ObjectInstance, UiInstance};
use wad_parser::{Level, SectorState, Ui};
use winit::window::Window;

use crate::graphics::GraphicsFlags;

use super::GraphicsContext;

const FOV_ANGLE: f32 = 90.0;

impl GraphicsContext {
	#[allow(clippy::too_many_arguments)]
	pub fn render(
		&mut self,
		window: &Window,
		world: &World,
		player_entity: Entity,
		level: &Level,
		game_state: GameState,
		global_timer: u32,
		alpha: f32,
	) {
		let obj_instances =
			self.collect_object_instances(world, player_entity, &level.state.sectors, alpha);
		let ui_instances = self.collect_ui_instances(world, player_entity, game_state);

		let size = window.inner_size();
		if size.width == 0 || size.height == 0 {
			return;
		}

		let aspect_ratio = size.width as f32 / size.height as f32;
		let proj = Mat4::perspective_rh(FOV_ANGLE.to_radians(), aspect_ratio, 1.0, 10000.0);

		update_camera_from_player(&mut self.view_matrix, world, player_entity, alpha);

		let mvp = MVP {
			model: Mat4::IDENTITY.to_cols_array(),
			view: self.view_matrix.to_cols_array(),
			proj: proj.to_cols_array(),
		};

		self.renderer.pin().updateUiInstances(&ui_instances);
		self.renderer.pin().updateObjectInstances(&obj_instances);

		self.renderer
			.pin()
			.setCameraYaw(calculate_camera_yaw(&mvp.view));
		self.renderer.pin().setGlobalTimer(global_timer);

		self.renderer.pin().startFrame(&mvp);
		self.renderer.pin().drawLevel();
		self.renderer.pin().drawObjects();
		self.renderer.pin().drawUi();
		self.renderer.pin().endFrame();
	}

	fn collect_object_instances(
		&self,
		world: &World,
		player_entity: Entity,
		sectors: &[SectorState],
		alpha: f32,
	) -> Vec<ObjectInstance> {
		let pos = world.get::<&Position>(player_entity).unwrap();

		let lerped_x = pos.prev_x * (1.0 - alpha) + pos.x * alpha;
		let lerped_y = pos.prev_y * (1.0 - alpha) + pos.y * alpha;
		let lerped_z = pos.prev_z * (1.0 - alpha) + pos.z * alpha;
		let player_pos = Vec3::new(lerped_x, lerped_y, lerped_z);

		let sprite_offsets = &self.offsets;

		let process_entity = |(pos, rot, current_sector, anim): (
			&Position,
			&MonsterRotation,
			&CurrentSector,
			&SpriteAnimation,
		)| {
			let lerped_x = pos.prev_x * (1.0 - alpha) + pos.x * alpha;
			let lerped_y = pos.prev_y * (1.0 - alpha) + pos.y * alpha;
			let lerped_z = pos.prev_z * (1.0 - alpha) + pos.z * alpha;

			let monster_pos = Vec3::new(lerped_x, lerped_y, lerped_z);

			let monster_angle = match rot.move_dir {
				Some(dir) => (dir as u32) << 29,
				None => 0,
			};

			let to_player = player_pos - monster_pos;
			let angle_to_player = fast_atan2(to_player.x, to_player.z);

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
			let (left_offset, top_offset) =
				sprite_offsets[tex_id.0 as usize - MAX_SKY.get().unwrap()];

			let mut final_width = tex_width as f32;
			let mut final_left_offset = left_offset as f32;

			if need_flip {
				final_width = -final_width;
				final_left_offset = tex_width as f32 - final_left_offset;
			}

			let sector = &sectors[current_sector.0.0];
			let light_level = if anim.full_bright {
				255
			} else {
				sector.light.clamp(0, 255) as u32
			};

			ObjectInstance {
				pos: [lerped_x, lerped_y, lerped_z],
				sprite_offset: [
					final_left_offset,
					(top_offset + anim.top_offset_shift) as f32,
				],
				sprite_size: [final_width, tex_height as f32],
				light_level,
				texture_id: tex_id.0,
			}
		};

		let mut entities_query = world.query::<(
			&Position,
			&MonsterRotation,
			&CurrentSector,
			&SpriteAnimation,
		)>();
		let iter = entities_query.iter();

		let (lower_bound, _) = iter.size_hint();
		const PARALLEL_THRESHOLD: usize = 2000;

		if lower_bound < PARALLEL_THRESHOLD {
			iter.map(&process_entity).collect()
		} else {
			let entities_to_process = iter.collect::<Vec<_>>();

			let nested_instances = entities_to_process
				.into_par_iter()
				.with_thread_pool(micropool::split_by_threads())
				.map(process_entity)
				.collect_per_thread();

			let total_count: usize = nested_instances.iter().map(|v: &Vec<_>| v.len()).sum();

			let mut instances = Vec::with_capacity(total_count);

			for mut thread_vec in nested_instances {
				instances.append(&mut thread_vec);
			}

			instances
		}
	}

	fn collect_ui_instances(
		&mut self,
		world: &World,
		player_entity: Entity,
		game_state: GameState,
	) -> Vec<UiInstance> {
		match game_state {
			GameState::Level | GameState::Demoscreen => {
				if self.cached_stbar_ui.arms.is_empty() {
					get_stbar(world, player_entity, &mut self.cached_stbar_ui);
				} else {
					let inventory = world.get::<&PlayerInventory>(player_entity).unwrap();
					let stats = world.get::<&PlayerStats>(player_entity).unwrap();
					let hp = world.get::<&Health>(player_entity).unwrap();

					let mut checked = [false; 7];

					for ui_type in self.ui_to_update.drain(..) {
						if checked[ui_type as usize] {
							continue;
						}

						match ui_type {
							UpdatableUiType::Ammo => {
								update_ammo_ui(&inventory, &mut self.cached_stbar_ui.ammo)
							}
							UpdatableUiType::Hp => update_hp_ui(&hp, &mut self.cached_stbar_ui.hp),
							UpdatableUiType::Arms => update_arms_ui(
								&inventory.weapon_owned,
								&mut self.cached_stbar_ui.arms,
							),
							UpdatableUiType::Face => update_face_ui(&mut self.cached_stbar_ui.face),
							UpdatableUiType::Armor => {
								update_armor_ui(stats.armor_points, &mut self.cached_stbar_ui.armor)
							}
							UpdatableUiType::Keys => {
								update_keys_ui(&inventory.cards, &mut self.cached_stbar_ui.keys)
							}
							UpdatableUiType::TotalAmmo => update_total_ammo_ui(
								&inventory,
								&mut self.cached_stbar_ui.total_ammo,
							),
						}

						checked[ui_type as usize] = true;
					}
				}

				self.cached_stbar_ui
					.stbar
					.iter()
					.chain(&self.cached_stbar_ui.ammo)
					.chain(&self.cached_stbar_ui.hp)
					.chain(&self.cached_stbar_ui.arms)
					.chain(&self.cached_stbar_ui.face)
					.chain(&self.cached_stbar_ui.armor)
					.chain(&self.cached_stbar_ui.keys)
					.chain(&self.cached_stbar_ui.total_ammo)
					.map(|&engine_ui| self.engine_ui_to_instance(engine_ui))
					.collect()
			}
			_ => unreachable!(),
		}
	}

	fn engine_ui_to_instance(&self, engine_ui: (Ui, f32, f32)) -> UiInstance {
		let (ui, x, y) = engine_ui;
		let (tex_id, width, height) = self.ui_db[ui as usize].unwrap();

		UiInstance {
			pos: [x, y],
			sprite_size: [width as f32, height as f32],
			texture_id: tex_id.0,
		}
	}

	pub fn system(&mut self, graphics_buffer: &mut Vec<GraphicsCommand>, _global_timer: u32) {
		for command in graphics_buffer.drain(..) {
			match command {
				GraphicsCommand::Palette(idx) => {
					self.renderer.pin().setPaletteIndex(idx);
					return;
				}
				GraphicsCommand::FullBright => {
					self.renderer
						.pin()
						.setFlags(GraphicsFlags::FULL_BRIGHT.bits());
				}
			}
		}

		let current_palette = self.renderer.pin().getPaletteIndex();

		let renderer = self.renderer.pin();
		match current_palette {
			0 => {}
			// red
			1 => renderer.setPaletteIndex(0),
			2 => renderer.setPaletteIndex(1),
			3 => renderer.setPaletteIndex(2),
			4 => renderer.setPaletteIndex(3),
			5 => renderer.setPaletteIndex(4),
			6 => renderer.setPaletteIndex(5),
			7 => renderer.setPaletteIndex(6),
			8 => renderer.setPaletteIndex(7),
			// bonuses
			9 => renderer.setPaletteIndex(0),
			10 => renderer.setPaletteIndex(9),
			11 => renderer.setPaletteIndex(10),
			12 => renderer.setPaletteIndex(11),
			// rad suit
			13 => {}
			_ => unreachable!(),
		}
	}
}

fn update_camera_from_player(
	view_matrix: &mut Mat4,
	world: &World,
	player_entity: Entity,
	alpha: f32,
) {
	let rot = world.get::<&PlayerRotation>(player_entity).unwrap();
	let pos = world.get::<&Position>(player_entity).unwrap();

	let prev_pos = glam::vec3(pos.prev_x, pos.prev_y + EYEHEIGHT, pos.prev_z);
	let current_pos = glam::vec3(pos.x, pos.y + EYEHEIGHT, pos.z);
	let interpolated_pos = prev_pos + (current_pos - prev_pos) * alpha;

	let angle_diff = rot.angle.wrapping_sub(rot.prev_angle) as i32;
	let interpolated_diff = (angle_diff as f64 * alpha as f64) as i32;
	let interpolated_angle_u32 = rot.prev_angle.wrapping_add_signed(interpolated_diff);

	let angle_normalized = interpolated_angle_u32 as f64 / u32::MAX as f64;
	let angle_rad = (angle_normalized * TAU) as f32;

	let target_dir = glam::vec3(f32::sin(angle_rad), 0.0, f32::cos(angle_rad));
	let camera_target = interpolated_pos + target_dir;

	let camera_up = glam::vec3(0.0, 1.0, 0.0);

	*view_matrix = Mat4::look_at_rh(interpolated_pos, camera_target, camera_up);
}

fn calculate_camera_yaw(view_matrix_array: &[f32; 16]) -> f32 {
	let yaw_u32 = fast_atan2(view_matrix_array[10], view_matrix_array[2]);

	(yaw_u32 as f64 / u32::MAX as f64 * TAU) as f32
}

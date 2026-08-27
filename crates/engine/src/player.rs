use crate::{
	CurrentSector, NUMAMMO, NUMCARDS, NUMWEAPONS, PlayerMarker, PlayerRotation, PlayerState,
	Position, Velocity, WeaponType,
};
use hecs::{Entity, World};
use std::f64::consts::TAU;
use wad_parser::Level;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerCamera {
	pub view_z: f32,
	pub view_height: f32,
	pub delta_view_height: f32,
	pub bob: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStats {
	pub state: PlayerState,
	pub armor_points: u32,
	pub is_super_armor: bool,
	pub kill_count: i32,
	pub item_count: i32,
	pub secret_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerInventory {
	pub ready_weapon: WeaponType,
	pub pending_weapon: WeaponType,
	pub backpack: bool,
	pub cards: [bool; NUMCARDS],
	pub weapon_owned: [bool; NUMWEAPONS],
	pub ammo: [u32; NUMAMMO],
}

impl Default for PlayerInventory {
	fn default() -> Self {
		PlayerInventory {
			ready_weapon: WeaponType::Pistol,
			pending_weapon: WeaponType::NoChange,
			backpack: false,
			cards: [false; NUMCARDS],
			weapon_owned: [true, true, false, false, false, false, false, false, false],
			ammo: [50, 0, 0, 0],
		}
	}
}

impl Default for PlayerStats {
	fn default() -> Self {
		PlayerStats {
			state: PlayerState::Live,
			armor_points: 0,
			is_super_armor: false,
			kill_count: 0,
			item_count: 0,
			secret_count: 0,
		}
	}
}

#[derive(Clone, Copy, Default)]
pub struct PlayerInput {
	pub move_forward: bool,
	pub move_backward: bool,
	pub move_left: bool,
	pub move_right: bool,
	pub move_up: bool,
	pub move_down: bool,
	pub shoot: bool,
	pub switch_fist_chainsaw: bool,
	pub choose_pistol: bool,
	pub choose_shotgun: bool,
	pub choose_chaingun: bool,
	pub choose_rlauncher: bool,
	pub choose_plasma: bool,
	pub choose_bfg: bool,
	pub mouse_delta_x: f32,
}

pub fn handle_position_input(world: &World, player_ent: Entity, input: &PlayerInput) {
	let rotation = world.get::<&PlayerRotation>(player_ent).unwrap();
	let mut velocity = world.get::<&mut Velocity>(player_ent).unwrap();

	let mut move_forward = 0.0;
	let mut move_sideways = 0.0;
	let mut move_vertically = 0.0;

	if input.move_forward {
		move_forward += 1.0;
	}
	if input.move_backward {
		move_forward -= 1.0;
	}
	if input.move_right {
		move_sideways += 1.0;
	}
	if input.move_left {
		move_sideways -= 1.0;
	}
	if input.move_up {
		move_vertically += 1.0;
	}
	if input.move_down {
		move_vertically -= 1.0;
	}

	let current_angle_rad = (rotation.angle as f64 / u32::MAX as f64) * TAU;

	let sin = f64::sin(current_angle_rad);
	let cos = f64::cos(current_angle_rad);

	let speed = 8.0;

	let thrust_x = (cos * move_sideways + sin * move_forward) * speed;
	let thrust_z = (-sin * move_sideways + cos * move_forward) * speed;

	velocity.x += thrust_x as f32 * 0.2;
	velocity.z += thrust_z as f32 * 0.2;
	velocity.y += move_vertically * 4.0;
}

pub fn handle_rotation_input(world: &World, player_ent: Entity, input: &PlayerInput) {
	let mut rotation = world.get::<&mut PlayerRotation>(player_ent).unwrap();

	let sensitivity = 0.008;
	let angle_delta_rad = input.mouse_delta_x * sensitivity;
	let factor = (angle_delta_rad as f64) / TAU;

	let angle_delta = (factor * u32::MAX as f64) as i32;

	rotation.prev_angle = rotation.angle;
	rotation.angle = rotation.angle.wrapping_add_signed(angle_delta);
}

pub fn apply_player_movement_system(world: &World, map: &Level) {
	let mut query = world
		.query::<(&mut Position, &Velocity, &mut CurrentSector)>()
		.with::<&PlayerMarker>();
	for (pos, velocity, current_sector) in query.iter() {
		pos.prev_x = pos.x;
		pos.prev_y = pos.y;
		pos.prev_z = pos.z;
		pos.x += velocity.x;
		pos.y += velocity.y;
		pos.z += velocity.z;

		current_sector.0 = map.get_sector_by_pos(pos.x, pos.z);
	}
}

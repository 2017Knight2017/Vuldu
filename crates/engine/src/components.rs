use crate::{
	enums::{MobjType, PlayerState},
	constants::{NUMCARDS, NUMWEAPONS, NUMAMMO}
};
use hecs::Bundle;

pub struct Transform {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub prev_x: f32,
	pub prev_y: f32,
	pub prev_z: f32,
	pub angle: u32,
    pub prev_angle: u32,
}

pub struct Velocity {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

pub struct Health(pub i32);

pub struct ActorState {
    pub mobj_type: MobjType,
    pub current_state_idx: usize,
    pub tics: i32,
    pub flags: i32,
}

pub struct Speed {
	pub default: u32,
	pub nightmare: Option<u32>
}

pub struct Damage {
	pub melee: Option<u32>,
	pub far: Option<u32>,
	pub nightmare: Option<u32>
}

pub struct BoundingBox {
    pub radius: f32,
    pub height: f32,
}

pub struct PhysicsEnvironment {
    pub floor_z: f32,
    pub ceiling_z: f32,
}

pub struct PlayerMarker;

pub struct PlayerCamera {
    pub view_z: f32,
    pub view_height: f32,
    pub delta_view_height: f32,
    pub bob: f32,
}

pub struct PlayerStats {
    pub state: PlayerState,
    pub armor_points: i32,
    pub armor_type: i32,
    pub kill_count: i32,
    pub item_count: i32,
    pub secret_count: i32,
}

pub struct PlayerInventory {
    pub ready_weapon: u32,
    pub pending_weapon: u32,
    pub backpack: bool,
    pub cards: [bool; NUMCARDS],
    pub weapon_owned: [bool; NUMWEAPONS],
	pub ammo: [i32; NUMAMMO],
    pub max_ammo: [i32; NUMAMMO],
}

pub struct WeaponOverlay {
    pub state_idx: u32,
    pub tics: i32,
    pub sx: f32,
    pub sy: f32,
}

#[derive(Bundle)]
pub struct MobjBundle {
    pub transform: Transform,
    pub velocity: Velocity,
    pub bbox: BoundingBox,
    pub env: PhysicsEnvironment,
    pub health: Health,
    pub state: ActorState,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub transform: Transform,
    pub velocity: Velocity,
    pub bbox: BoundingBox,
    pub env: PhysicsEnvironment,
    pub health: Health,
    pub state: ActorState,

    pub marker: PlayerMarker,
    pub camera: PlayerCamera,
    pub stats: PlayerStats,
    pub inventory: PlayerInventory,
    pub weapon_overlay: WeaponOverlay,
}

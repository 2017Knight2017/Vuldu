use crate::{NUMAMMO, NUMCARDS, NUMWEAPONS, PlayerRotation, PlayerShoot, PlayerState, SfxEvent, Velocity};
use std::f64::consts::TAU;
use hecs::{CommandBuffer, Entity, World};
use wad_parser::to_u64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerCamera {
    pub view_z: f32,
    pub view_height: f32,
    pub delta_view_height: f32,
    pub bob: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerStats {
    pub state: PlayerState,
    pub armor_points: i32,
    pub armor_type: i32,
    pub kill_count: i32,
    pub item_count: i32,
    pub secret_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerInventory {
    pub ready_weapon: u32,
    pub pending_weapon: u32,
    pub backpack: bool,
    pub cards: [bool; NUMCARDS],
    pub weapon_owned: [bool; NUMWEAPONS],
	pub ammo: [i32; NUMAMMO],
    pub max_ammo: [i32; NUMAMMO],
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
    pub mouse_delta_x: f32,
}

pub fn handle_position_input(
    world: &World,
    input: &PlayerInput,
    command_buffer: &mut CommandBuffer,
    audio_buffer: &mut Vec<SfxEvent>
) {
    let mut query = world.query::<(Entity, &mut Velocity, &PlayerRotation)>();
    for (entity, velocity, rotation) in query.iter() {
        let mut move_forward = 0.0;
        let mut move_sideways = 0.0;
        let mut move_vertically = 0.0;

        if input.move_forward  { move_forward += 1.0; }
        if input.move_backward { move_forward -= 1.0; }
        if input.move_right    { move_sideways += 1.0; }
        if input.move_left     { move_sideways -= 1.0; }
        if input.move_up       { move_vertically += 1.0; }
        if input.move_down     { move_vertically -= 1.0; }

        let current_angle_rad = (rotation.angle as f64 / u32::MAX as f64) * TAU;

        let sin = f64::sin(current_angle_rad);
        let cos = f64::cos(current_angle_rad);

        let speed = 8.0;

        let thrust_x = (cos * move_sideways + sin * move_forward) * speed;
        let thrust_z = (-sin * move_sideways + cos * move_forward) * speed;

        velocity.x += thrust_x as f32 * 0.2; 
        velocity.z += thrust_z as f32 * 0.2;
        velocity.y += move_vertically * 4.0;

        if input.shoot{
            command_buffer.insert_one(entity, PlayerShoot);
            audio_buffer.push(SfxEvent { sfx_id: to_u64(b"DSPISTOL"), pos: None });
        }
    }
}

pub fn handle_rotation_input(world: &World, input: &PlayerInput) {
    let mut query = world.query::<&mut PlayerRotation>();
    for rotation in query.iter() {
        let sensitivity = 0.008; 
        let angle_delta_rad = input.mouse_delta_x * sensitivity;
        let factor = (angle_delta_rad as f64) / TAU;

        let angle_delta = (factor * u32::MAX as f64) as i32;

        rotation.prev_angle = rotation.angle;
        rotation.angle = rotation.angle.wrapping_add_signed(angle_delta);
    }
}

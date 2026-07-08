use crate::{
	components::{Velocity, Transform, PlayerMarker, SpriteAnimation},
	constants::FRICTION,
    data_tables::DB
};
use std::f64::consts::TAU;
use hecs::World;

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

pub fn update_physics(world: &mut World, input: &PlayerInput) {
    for (velocity, transform, _player) in world.query_mut::<(&mut Velocity, &mut Transform, &PlayerMarker)>() {
        
        let mut move_forward = 0.0;
        let mut move_sideways = 0.0;
        let mut move_vertically = 0.0;

        if input.move_forward  { move_forward += 1.0; }
        if input.move_backward { move_forward -= 1.0; }
        if input.move_left     { move_sideways += 1.0; }
        if input.move_right    { move_sideways -= 1.0; }
        if input.move_up       { move_vertically += 1.0; }
        if input.move_down     { move_vertically -= 1.0; }

        let current_angle_rad = (transform.angle as f64 / u32::MAX as f64) * TAU;

        let sin = f64::sin(current_angle_rad);
        let cos = f64::cos(current_angle_rad);

        let speed = 8.0;

        let thrust_x = (cos * move_sideways + sin * move_forward) * speed;
        let thrust_z = (-sin * move_sideways + cos * move_forward) * speed;

        velocity.x += thrust_x as f32 * 0.2; 
        velocity.z += thrust_z as f32 * 0.2;
        velocity.y += move_vertically * 4.0;

		let sensitivity = 0.008; 
        let angle_delta_rad = -input.mouse_delta_x * sensitivity;
        let factor = (angle_delta_rad as f64) / TAU;

        let angle_delta = (factor * u32::MAX as f64) as i32;

        transform.prev_angle = transform.angle;
        transform.angle = transform.angle.wrapping_add_signed(angle_delta);
    }
}

pub fn system_movement_and_friction(world: &mut World) {
	for (transform, velocity) in world.query_mut::<(&mut Transform, &mut Velocity)>() {
		velocity.x *= FRICTION;
		velocity.y *= 0.7;
		velocity.z *= FRICTION;

		transform.prev_x = transform.x;
        transform.prev_y = transform.y;
		transform.prev_z = transform.z;
        transform.x += velocity.x;
        transform.y += velocity.y;
        transform.z += velocity.z;
    }
}

pub fn animation_system(world: &mut World) {
    for anim in world.query_mut::<&mut SpriteAnimation>() {
        if anim.tics_left <= 0 {
            continue; 
        }

        anim.tics_left -= 1;
        if anim.tics_left != 0 { 
            continue; 
        }

        if let (Some(db), Some(current_state)) = (DB.get(), anim.current_state) {
            let current_state_data = db.states[&current_state];

            if let Some(next_state_id) = current_state_data.next_state {
                let next_state_data = db.states[&next_state_id];
                
                anim.current_state = Some(next_state_id);
                anim.tics_left = next_state_data.tics;
                anim.cached_rotations = next_state_data.cached_rotations;
            }
        }
    }
}

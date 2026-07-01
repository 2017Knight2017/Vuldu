use crate::{
	constants::{NUMCARDS, NUMWEAPONS},
	components::*,
	enums::*
};
use hecs::World;

pub fn spawn_player(world: &mut World, x_raw: i16, y_raw: i16, z_raw: i16, angle_raw: i16) {
	let x = x_raw as f32;
	let y = y_raw as f32;
	let z = z_raw as f32;

    let angle = angle_raw as u32 / 45 * 0x20000000;

	let _ = world.spawn(PlayerBundle {
	    transform: Transform { x, y, z, prev_x: x, prev_y: y, prev_z: z, angle, prev_angle: angle },
	    velocity: Velocity { x: 0.0, y: 0.0, z: 0.0 },
	    bbox: BoundingBox { radius: 16.0, height: 56.0 },
	    env: PhysicsEnvironment { floor_z: y, ceiling_z: 128.0 },
	    health: Health(100),
	    state: ActorState { mobj_type: MobjType::Player, current_state_idx: 1, tics: 0, flags: 0 },
		
	    marker: PlayerMarker,
	    camera: PlayerCamera { view_z: 41.0, view_height: 41.0, delta_view_height: 0.0, bob: 0.0 },
	    stats: PlayerStats { state: PlayerState::Live, armor_points: 0, armor_type: 0, kill_count: 0, item_count: 0, secret_count: 0 },
	    inventory: PlayerInventory { ready_weapon: 1, pending_weapon: 1, backpack: false, cards: [false; NUMCARDS], weapon_owned: [false; NUMWEAPONS], ammo: [50, 0, 0, 0], max_ammo: [200, 50, 50, 300] },
	    weapon_overlay: WeaponOverlay { state_idx: 0, tics: 0, sx: 0.0, sy: 0.0 },
	});
}
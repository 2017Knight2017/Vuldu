use hecs::{CommandBuffer, Entity, World};
use serde::Deserialize;
use strum::EnumIter;
use wad_parser::to_u64;

use crate::{PlayerInput, PlayerInventory, PlayerShoot, SfxEvent, UpdatableUiType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum WeaponType {
	Fist,
	Pistol,
	Shotgun,
	Chaingun,
	Missile,
	Plasma,
	BFG,
	Chainsaw,
	SuperShotgun,
	NoChange,
}

impl WeaponType {
	fn ammo_spent_per_shot(&self) -> u32 {
		match *self {
			WeaponType::BFG => 40,
			WeaponType::SuperShotgun => 2,
			_ => 1,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, EnumIter)]
pub enum AmmoType {
	Clip,
	Shell,
	Missile,
	Cell,
	NoAmmo,
}

impl From<WeaponType> for AmmoType {
	fn from(value: WeaponType) -> Self {
		match value {
			WeaponType::Pistol | WeaponType::Chaingun => AmmoType::Clip,
			WeaponType::Shotgun | WeaponType::SuperShotgun => AmmoType::Shell,
			WeaponType::Missile => AmmoType::Missile,
			WeaponType::Plasma | WeaponType::BFG => AmmoType::Cell,
			WeaponType::Fist | WeaponType::Chainsaw | WeaponType::NoChange => AmmoType::NoAmmo,
		}
	}
}

pub fn handle_weapons_input(
	world: &World,
	player_entity: Entity,
	ui_to_update: &mut Vec<UpdatableUiType>,
	command_buffer: &mut CommandBuffer,
	audio: &mut Vec<SfxEvent>,
	input: &PlayerInput,
) {
	let mut inventory = world.get::<&mut PlayerInventory>(player_entity).unwrap();
	let previous_ready_weapon = inventory.ready_weapon;

	if input.switch_fist_chainsaw {
		if inventory.ready_weapon == WeaponType::Chainsaw
			|| !is_available(WeaponType::Chainsaw, &inventory)
		{
			inventory.ready_weapon = WeaponType::Fist;
		} else {
			inventory.ready_weapon = WeaponType::Chainsaw;
		}
	} else if input.switch_fist_chainsaw && inventory.ready_weapon == WeaponType::Chainsaw {
		inventory.ready_weapon = WeaponType::Fist;
	} else if input.choose_pistol && is_available(WeaponType::Pistol, &inventory) {
		inventory.ready_weapon = WeaponType::Pistol;
	} else if input.choose_shotgun && is_available(WeaponType::SuperShotgun, &inventory) {
		inventory.ready_weapon = if inventory.ready_weapon == WeaponType::SuperShotgun {
			WeaponType::Shotgun
		} else {
			WeaponType::SuperShotgun
		};
	} else if input.choose_shotgun && is_available(WeaponType::Shotgun, &inventory) {
		inventory.ready_weapon = WeaponType::Shotgun;
	} else if input.choose_chaingun && is_available(WeaponType::Chaingun, &inventory) {
		inventory.ready_weapon = WeaponType::Chaingun;
	} else if input.choose_rlauncher && is_available(WeaponType::Missile, &inventory) {
		inventory.ready_weapon = WeaponType::Missile;
	} else if input.choose_plasma && is_available(WeaponType::Plasma, &inventory) {
		inventory.ready_weapon = WeaponType::Plasma;
	} else if input.choose_bfg && is_available(WeaponType::BFG, &inventory) {
		inventory.ready_weapon = WeaponType::BFG;
	}

	if previous_ready_weapon != inventory.ready_weapon {
		ui_to_update.push(UpdatableUiType::Ammo);
	}

	if input.shoot && inventory.pending_weapon == WeaponType::NoChange {
		let ammo_per_shot = inventory.ready_weapon.ammo_spent_per_shot();
		let ammo_type = AmmoType::from(inventory.ready_weapon);

		if ammo_type != AmmoType::NoAmmo {
			let ammo_remained = inventory.ammo[ammo_type as usize];

			if ammo_per_shot > ammo_remained {
				inventory.ready_weapon = choose_best(&inventory);
				ui_to_update.push(UpdatableUiType::Ammo);
				return;
			}

			inventory.ammo[ammo_type as usize] -= ammo_per_shot;

			ui_to_update.push(UpdatableUiType::Ammo);
			ui_to_update.push(UpdatableUiType::TotalAmmo);
		}

		command_buffer.insert_one(player_entity, PlayerShoot);

		let sfx_id = match inventory.ready_weapon {
			WeaponType::Fist => to_u64(b"DSPUNCH"),
			WeaponType::SuperShotgun => to_u64(b"DSDSHTGN"),
			WeaponType::Shotgun => to_u64(b"DSSHOTGN"),
			WeaponType::Pistol | WeaponType::Chaingun => to_u64(b"DSPISTOL"),
			WeaponType::Chainsaw => to_u64(b"DSSAWUP"),
			WeaponType::Missile => to_u64(b"DSRLAUNC"),
			WeaponType::Plasma => to_u64(b"DSPLASMA"),
			WeaponType::BFG => to_u64(b"DSBFG"),
			WeaponType::NoChange => unreachable!(),
		};

		audio.push(SfxEvent { sfx_id, pos: None });
	}
}

fn choose_best(inventory: &PlayerInventory) -> WeaponType {
	if is_available(WeaponType::Plasma, inventory) {
		WeaponType::Plasma
	} else if is_available(WeaponType::SuperShotgun, inventory) {
		WeaponType::SuperShotgun
	} else if is_available(WeaponType::Chaingun, inventory) {
		WeaponType::Chaingun
	} else if is_available(WeaponType::Shotgun, inventory) {
		WeaponType::Shotgun
	} else if is_available(WeaponType::Pistol, inventory) {
		WeaponType::Pistol
	} else if is_available(WeaponType::Chainsaw, inventory) {
		WeaponType::Chainsaw
	} else if is_available(WeaponType::Missile, inventory) {
		WeaponType::Missile
	} else if is_available(WeaponType::BFG, inventory) {
		WeaponType::BFG
	} else {
		WeaponType::Fist
	}
}

fn is_available(weapon: WeaponType, inventory: &PlayerInventory) -> bool {
	let ammo_type = AmmoType::from(weapon);
	if ammo_type == AmmoType::NoAmmo {
		inventory.weapon_owned[weapon as usize]
	} else {
		inventory.weapon_owned[weapon as usize]
			&& inventory.ammo[AmmoType::from(weapon) as usize] >= weapon.ammo_spent_per_shot()
	}
}

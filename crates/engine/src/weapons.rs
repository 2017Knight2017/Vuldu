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
pub(crate) enum AmmoType {
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
	input: PlayerInput,
) {
	let mut inv = world.get::<&mut PlayerInventory>(player_entity).unwrap();
	let previous_ready_weapon = inv.ready_weapon;

	if input.switch_fist_chainsaw {
		if inv.ready_weapon == WeaponType::Chainsaw || !is_available(WeaponType::Chainsaw, *inv) {
			inv.ready_weapon = WeaponType::Fist;
		} else {
			inv.ready_weapon = WeaponType::Chainsaw;
		}
	} else if input.switch_fist_chainsaw && inv.ready_weapon == WeaponType::Chainsaw {
		inv.ready_weapon = WeaponType::Fist;
	} else if input.choose_pistol && is_available(WeaponType::Pistol, *inv) {
		inv.ready_weapon = WeaponType::Pistol;
	} else if input.choose_shotgun && is_available(WeaponType::SuperShotgun, *inv) {
		inv.ready_weapon = if inv.ready_weapon == WeaponType::SuperShotgun {
			WeaponType::Shotgun
		} else {
			WeaponType::SuperShotgun
		};
	} else if input.choose_shotgun && is_available(WeaponType::Shotgun, *inv) {
		inv.ready_weapon = WeaponType::Shotgun;
	} else if input.choose_chaingun && is_available(WeaponType::Chaingun, *inv) {
		inv.ready_weapon = WeaponType::Chaingun;
	} else if input.choose_rlauncher && is_available(WeaponType::Missile, *inv) {
		inv.ready_weapon = WeaponType::Missile;
	} else if input.choose_plasma && is_available(WeaponType::Plasma, *inv) {
		inv.ready_weapon = WeaponType::Plasma;
	} else if input.choose_bfg && is_available(WeaponType::BFG, *inv) {
		inv.ready_weapon = WeaponType::BFG;
	}

	if previous_ready_weapon != inv.ready_weapon {
		ui_to_update.push(UpdatableUiType::Ammo);
	}

	if input.shoot && inv.pending_weapon == WeaponType::NoChange {
		let ammo_per_shot = inv.ready_weapon.ammo_spent_per_shot();
		let ammo_type = AmmoType::from(inv.ready_weapon);

		if ammo_type != AmmoType::NoAmmo {
			let ammo_remained = inv.ammo[ammo_type as usize];

			if ammo_per_shot > ammo_remained {
				inv.ready_weapon = choose_best(*inv);
				ui_to_update.push(UpdatableUiType::Ammo);
				return;
			}

			inv.ammo[ammo_type as usize] -= ammo_per_shot;

			ui_to_update.push(UpdatableUiType::Ammo);
			ui_to_update.push(UpdatableUiType::TotalAmmo);
		}

		command_buffer.insert_one(player_entity, PlayerShoot);

		let sfx_id = match inv.ready_weapon {
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

fn choose_best(inv: PlayerInventory) -> WeaponType {
	if is_available(WeaponType::Plasma, inv) {
		WeaponType::Plasma
	} else if is_available(WeaponType::SuperShotgun, inv) {
		WeaponType::SuperShotgun
	} else if is_available(WeaponType::Chaingun, inv) {
		WeaponType::Chaingun
	} else if is_available(WeaponType::Shotgun, inv) {
		WeaponType::Shotgun
	} else if is_available(WeaponType::Pistol, inv) {
		WeaponType::Pistol
	} else if is_available(WeaponType::Chainsaw, inv) {
		WeaponType::Chainsaw
	} else if is_available(WeaponType::Missile, inv) {
		WeaponType::Missile
	} else if is_available(WeaponType::BFG, inv) {
		WeaponType::BFG
	} else {
		WeaponType::Fist
	}
}

fn is_available(weapon: WeaponType, inv: PlayerInventory) -> bool {
	let ammo_type = AmmoType::from(weapon);
	if ammo_type == AmmoType::NoAmmo {
		inv.weapon_owned[weapon as usize]
	} else {
		inv.weapon_owned[weapon as usize]
			&& inv.ammo[AmmoType::from(weapon) as usize] >= weapon.ammo_spent_per_shot()
	}
}

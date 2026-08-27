use hecs::{CommandBuffer, Entity, World};
use wad_parser::{Level, to_u64};

use crate::{
	AmmoType, Card, GameConfig, Health, MobjFlags, MobjNum, MobjType, NUMCARDS, NUMWEAPONS,
	PICKUP_MESSAGES, PlayerInventory, PlayerStats, SfxEvent, SkillLevel, UpdatableUiType,
	WeaponType, kill_mobj,
};

#[derive(Debug)]
pub enum WorldEvent {
	DamageMobj {
		target: Entity,
		inflictor: Entity,
		damage: u32,
	},
	ResetSkullFly {
		actor_id: Entity,
	},
	TouchSpecialThing {
		special_item: Entity,
		picker: Entity,
	},
	CheatIDKFA,
	CheatIDFA,
	CheatIDDQD,
	CheatNOCLIP,
}

#[allow(clippy::too_many_arguments)]
pub fn execute_events_system(
	world_events: &mut Vec<WorldEvent>,
	world: &World,
	level: &Level,
	player_ent: Entity,
	ui_to_update: &mut Vec<UpdatableUiType>,
	cmd: &mut CommandBuffer,
	audio_buffer: &mut Vec<SfxEvent>,
	blocklists: &mut [Vec<Entity>],
	cfg: &GameConfig,
	global_timer: u32,
) {
	for event in world_events.drain(..) {
		match event {
			WorldEvent::DamageMobj {
				target: _,
				inflictor: _,
				damage: _,
			} => {}
			WorldEvent::ResetSkullFly { actor_id: _ } => {}
			WorldEvent::TouchSpecialThing {
				special_item,
				picker,
			} => {
				let Ok(mut inventory) = world.get::<&mut PlayerInventory>(picker) else {
					continue;
				};
				let Ok(mut stats) = world.get::<&mut PlayerStats>(picker) else {
					continue;
				};
				let Ok(mut hp) = world.get::<&mut Health>(picker) else {
					continue;
				};

				let Ok(item_type) = world.get::<&MobjType>(special_item) else {
					continue;
				};

				if let Ok(()) = special_item_effect(
					&item_type,
					&mut inventory,
					&mut stats,
					&mut hp,
					ui_to_update,
					audio_buffer,
					cfg,
					global_timer,
				) {
					if item_type.type_ == MobjNum::Misc11 && hp.0 < 50 {
						println!("Picked up a medikit that you REALLY need!")
					} else {
						println!(
							"{}",
							PICKUP_MESSAGES[item_type.type_ as usize - MobjNum::Misc0 as usize]
						);
					}

					kill_mobj(special_item, world, level, cmd, blocklists);
				}
			}
			WorldEvent::CheatIDKFA => {
				let mut inventory = world.get::<&mut PlayerInventory>(player_ent).unwrap();

				inventory.backpack = true;
				inventory.ammo = [400, 100, 100, 600];
				inventory.weapon_owned = [true; NUMWEAPONS];
				inventory.cards = [true; NUMCARDS];

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::Arms);
				ui_to_update.push(UpdatableUiType::Face);
				ui_to_update.push(UpdatableUiType::Keys);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				println!("Very Happy Ammo Added");
			}
			WorldEvent::CheatIDFA => {
				let mut inventory = world.get::<&mut PlayerInventory>(player_ent).unwrap();

				inventory.backpack = true;
				inventory.ammo = [400, 100, 100, 600];
				inventory.weapon_owned = [true; NUMWEAPONS];

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::Arms);
				ui_to_update.push(UpdatableUiType::Face);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				println!("Ammo (no keys) Added");
			}
			WorldEvent::CheatIDDQD => {}
			WorldEvent::CheatNOCLIP => {}
		}
	}
}

#[allow(clippy::too_many_arguments)]
fn special_item_effect(
	mobj_type: &MobjType,
	inventory: &mut PlayerInventory,
	stats: &mut PlayerStats,
	hp: &mut Health,
	ui_to_update: &mut Vec<UpdatableUiType>,
	audio_buffer: &mut Vec<SfxEvent>,
	cfg: &GameConfig,
	global_timer: u32,
) -> Result<(), ()> {
	const CLIP_IDX: usize = AmmoType::Clip as usize;
	const MISSILE_IDX: usize = AmmoType::Missile as usize;
	const CELL_IDX: usize = AmmoType::Cell as usize;
	const SHELL_IDX: usize = AmmoType::Shell as usize;

	let max_ammo = [200, 50, 50, 300];
	let skill_mult = (cfg.skill == SkillLevel::Baby || cfg.skill == SkillLevel::Nightmare) as u32;
	let from_monster_divisor = mobj_type.flags.contains(MobjFlags::DROPPED) as u32;

	match mobj_type.type_ {
		MobjNum::Misc0 => {
			if stats.armor_points < 100 {
				stats.armor_points = 100;
				stats.is_super_armor = false;

				ui_to_update.push(UpdatableUiType::Armor);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc1 => {
			if stats.armor_points < 200 {
				stats.armor_points = 200;
				stats.is_super_armor = true;

				ui_to_update.push(UpdatableUiType::Armor);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc2 => {
			hp.0 = (hp.0 + 1).min(200);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			ui_to_update.push(UpdatableUiType::Face);

			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});
			Ok(())
		}
		MobjNum::Misc3 => {
			stats.armor_points = (stats.armor_points + 1).min(200);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Armor);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc4 => {
			inventory.cards[Card::BlueCard as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc5 => {
			inventory.cards[Card::RedCard as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc6 => {
			inventory.cards[Card::YellowCard as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc7 => {
			inventory.cards[Card::YellowSkull as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc8 => {
			inventory.cards[Card::RedSkull as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc9 => {
			inventory.cards[Card::BlueSkull as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc10 => {
			if hp.0 < 100 {
				hp.0 = (hp.0 + 10).min(100);

				ui_to_update.push(UpdatableUiType::Hp);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc11 => {
			if hp.0 < 100 {
				hp.0 = (hp.0 + 25).min(100);

				ui_to_update.push(UpdatableUiType::Hp);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc12 => {
			hp.0 = (hp.0 + 100).min(200);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc13 => {
			if hp.0 < 100 {
				hp.0 = 100;
			}
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});
			stats.berserk_timestamp = Some(global_timer);

			Ok(())
		}
		MobjNum::Misc14 => {
			stats.rad_suit_timestamp = Some(global_timer);

			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});
			Ok(())
		}
		MobjNum::Misc15 => {
			if !stats.computer_area_map {
				stats.computer_area_map = true;
				stats.item_count += 1;

				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSGETPOW"),
					pos: None,
				});
				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc16 => {
			stats.goggles_timestamp = Some(global_timer);
			stats.item_count += 1;

			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Inv => {
			stats.invuln_timestamp = Some(global_timer);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Face);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Ins => {
			// TODO: make the player's weapon partially invisible
			stats.invis_timestamp = Some(global_timer);
			stats.item_count += 1;

			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Mega => {
			hp.0 = 200;
			stats.armor_points = 200;
			stats.is_super_armor = true;
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			ui_to_update.push(UpdatableUiType::Armor);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Clip => {
			let max_ammo = max_ammo[CLIP_IDX] << inventory.backpack as u32;

			if inventory.ammo[CLIP_IDX] < max_ammo {
				let new_ammo =
					inventory.ammo[CLIP_IDX] + (10 << skill_mult >> from_monster_divisor);
				inventory.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc17 => {
			let max_ammo = max_ammo[CLIP_IDX] << inventory.backpack as u32;

			if inventory.ammo[CLIP_IDX] < max_ammo {
				let new_ammo = inventory.ammo[CLIP_IDX] + (50 << skill_mult);
				inventory.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc18 => {
			let max_ammo = max_ammo[MISSILE_IDX] << inventory.backpack as u32;

			if inventory.ammo[MISSILE_IDX] < max_ammo {
				let new_ammo = inventory.ammo[MISSILE_IDX] + (1 << skill_mult);
				inventory.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc19 => {
			let max_ammo = max_ammo[MISSILE_IDX] << inventory.backpack as u32;

			if inventory.ammo[MISSILE_IDX] < max_ammo {
				let new_ammo = inventory.ammo[MISSILE_IDX] + (5 << skill_mult);
				inventory.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc20 => {
			let max_ammo = max_ammo[CELL_IDX] << inventory.backpack as u32;

			if inventory.ammo[CELL_IDX] < max_ammo {
				let new_ammo = inventory.ammo[CELL_IDX] + (20 << skill_mult);
				inventory.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc21 => {
			let max_ammo = max_ammo[CELL_IDX] << inventory.backpack as u32;

			if inventory.ammo[CELL_IDX] < max_ammo {
				let new_ammo = inventory.ammo[CELL_IDX] + (100 << skill_mult);
				inventory.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc22 => {
			let max_ammo = max_ammo[SHELL_IDX] << inventory.backpack as u32;

			if inventory.ammo[SHELL_IDX] < max_ammo {
				let new_ammo = inventory.ammo[SHELL_IDX] + (4 << skill_mult);
				inventory.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc23 => {
			let max_ammo = max_ammo[CLIP_IDX] << inventory.backpack as u32;

			if inventory.ammo[SHELL_IDX] < max_ammo {
				let new_ammo = inventory.ammo[SHELL_IDX] + (20 << skill_mult);
				inventory.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(())
			} else {
				Err(())
			}
		}
		MobjNum::Misc24 => {
			inventory.backpack = true;

			let new_clips = inventory.ammo[CLIP_IDX] + (10 << skill_mult);
			let new_missiles = inventory.ammo[MISSILE_IDX] + (1 << skill_mult);
			let new_cells = inventory.ammo[CELL_IDX] + (20 << skill_mult);
			let new_shells = inventory.ammo[SHELL_IDX] + (4 << skill_mult);

			inventory.ammo[CLIP_IDX] = new_clips.min(max_ammo[CLIP_IDX] << 1);
			inventory.ammo[MISSILE_IDX] = new_missiles.min(max_ammo[MISSILE_IDX] << 1);
			inventory.ammo[CELL_IDX] = new_cells.min(max_ammo[CELL_IDX] << 1);
			inventory.ammo[SHELL_IDX] = new_shells.min(max_ammo[SHELL_IDX] << 1);

			ui_to_update.push(UpdatableUiType::Ammo);
			ui_to_update.push(UpdatableUiType::TotalAmmo);
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(())
		}
		MobjNum::Misc25 => {
			let max_ammo = max_ammo[CELL_IDX] << inventory.backpack as u32;
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::BFG as usize] {
				inventory.weapon_owned[WeaponType::BFG as usize] = true;
				inventory.pending_weapon = WeaponType::BFG;

				if cfg.dmatch {
					let new_ammo = inventory.ammo[CELL_IDX] + 100;
					inventory.ammo[CELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inventory.ammo[CELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inventory.ammo[CELL_IDX] + (40 << skill_mult);
				inventory.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		MobjNum::Misc26 => {
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::Chainsaw as usize] {
				inventory.weapon_owned[WeaponType::Chainsaw as usize] = true;
				inventory.pending_weapon = WeaponType::Chainsaw;

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		MobjNum::Misc27 => {
			let max_ammo = max_ammo[MISSILE_IDX] << inventory.backpack as u32;
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::Missile as usize] {
				inventory.weapon_owned[WeaponType::Missile as usize] = true;
				inventory.pending_weapon = WeaponType::Missile;

				if cfg.dmatch {
					let new_ammo = inventory.ammo[MISSILE_IDX] + 5;
					inventory.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inventory.ammo[MISSILE_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inventory.ammo[MISSILE_IDX] + (2 << skill_mult);
				inventory.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		MobjNum::Misc28 => {
			let max_ammo = max_ammo[CELL_IDX] << inventory.backpack as u32;
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::Plasma as usize] {
				inventory.weapon_owned[WeaponType::Plasma as usize] = true;
				inventory.pending_weapon = WeaponType::Plasma;

				if cfg.dmatch {
					let new_ammo = inventory.ammo[CELL_IDX] + 100;
					inventory.ammo[CELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inventory.ammo[CELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inventory.ammo[CELL_IDX] + (40 << skill_mult);
				inventory.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		MobjNum::Chaingun => {
			let max_ammo = max_ammo[CLIP_IDX] << inventory.backpack as u32;
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::Chaingun as usize] {
				inventory.weapon_owned[WeaponType::Chaingun as usize] = true;
				inventory.pending_weapon = WeaponType::Chaingun;

				if cfg.dmatch {
					let new_ammo = inventory.ammo[CLIP_IDX] + 50;
					inventory.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inventory.ammo[CLIP_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo =
					inventory.ammo[CLIP_IDX] + (20 << skill_mult >> from_monster_divisor);
				inventory.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		MobjNum::Shotgun => {
			let max_ammo = max_ammo[SHELL_IDX] << inventory.backpack as u32;
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::Shotgun as usize] {
				inventory.weapon_owned[WeaponType::Shotgun as usize] = true;
				inventory.pending_weapon = WeaponType::Shotgun;

				if cfg.dmatch {
					let new_ammo = inventory.ammo[SHELL_IDX] + 20;
					inventory.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inventory.ammo[SHELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo =
					inventory.ammo[SHELL_IDX] + (8 << skill_mult >> from_monster_divisor);
				inventory.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		MobjNum::Supershotgun => {
			let max_ammo = max_ammo[SHELL_IDX] << inventory.backpack as u32;
			let mut success = false;

			if !inventory.weapon_owned[WeaponType::SuperShotgun as usize] {
				inventory.weapon_owned[WeaponType::SuperShotgun as usize] = true;
				inventory.pending_weapon = WeaponType::SuperShotgun;

				if cfg.dmatch {
					let new_ammo = inventory.ammo[SHELL_IDX] + 20;
					inventory.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inventory.ammo[SHELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inventory.ammo[SHELL_IDX] + (8 << skill_mult);
				inventory.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio_buffer.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Err(()) } else { Ok(()) }
			} else {
				Err(())
			}
		}
		_ => unreachable!(),
	}
}

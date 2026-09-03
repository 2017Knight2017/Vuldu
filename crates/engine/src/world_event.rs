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
		item_ent: Entity,
		picker: Entity,
	},
	CheatIDKFA,
	CheatIDFA,
	CheatIDDQD,
	CheatNOCLIP,
}

#[derive(PartialEq, Eq)]
pub enum GraphicsCommand {
	Palette(u32),
	FullBright,
}

#[allow(clippy::too_many_arguments)]
pub fn execute_events_system(
	world_events: &mut Vec<WorldEvent>,
	world: &World,
	level: &Level,
	player_ent: Entity,
	ui_to_update: &mut Vec<UpdatableUiType>,
	cmd: &mut CommandBuffer,
	audio: &mut Vec<SfxEvent>,
	blocklists: &mut [Vec<Entity>],
	graphics_buffer: &mut Vec<GraphicsCommand>,
	cfg: GameConfig,
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
			WorldEvent::TouchSpecialThing { item_ent, picker } => {
				let mut query = world
					.query_one::<(&mut PlayerInventory, &mut PlayerStats, &mut Health)>(picker);
				let Ok((inv, stats, hp)) = query.get() else {
					continue;
				};

				let Ok(item) = world.get::<&MobjType>(item_ent).map(|i| *i) else {
					continue;
				};

				match special_item_effect(
					item,
					inv,
					stats,
					hp,
					ui_to_update,
					audio,
					cfg,
					global_timer,
				) {
					Ok(Some(_)) => {
						if item.type_ == MobjNum::Misc11 && hp.0 < 25 {
							println!("Picked up a medikit that you REALLY need!")
						} else {
							println!(
								"{}",
								PICKUP_MESSAGES[item.type_ as usize - MobjNum::Misc0 as usize]
							);
						}

						if item.type_ == MobjNum::Misc16 {
							graphics_buffer.push(GraphicsCommand::FullBright);
						}

						graphics_buffer.push(GraphicsCommand::Palette(12));
						kill_mobj(item_ent, world, level, cmd, blocklists);
					}
					Ok(None) => {
						// if we're here, we've just picked up a weapon,
						// so we don't check for medikit.
						println!(
							"{}",
							PICKUP_MESSAGES[item.type_ as usize - MobjNum::Misc0 as usize]
						);
						graphics_buffer.push(GraphicsCommand::Palette(12));
					}
					Err(_) => {}
				}
			}
			WorldEvent::CheatIDKFA => {
				let mut inv = world.get::<&mut PlayerInventory>(player_ent).unwrap();

				inv.backpack = true;
				inv.ammo = [400, 100, 100, 600];
				inv.weapon_owned = [true; NUMWEAPONS];
				inv.cards = [true; NUMCARDS];

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::Arms);
				ui_to_update.push(UpdatableUiType::Face);
				ui_to_update.push(UpdatableUiType::Keys);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				println!("Very Happy Ammo Added");
			}
			WorldEvent::CheatIDFA => {
				let mut inv = world.get::<&mut PlayerInventory>(player_ent).unwrap();

				inv.backpack = true;
				inv.ammo = [400, 100, 100, 600];
				inv.weapon_owned = [true; NUMWEAPONS];

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
	mobj_type: MobjType,
	inv: &mut PlayerInventory,
	stats: &mut PlayerStats,
	hp: &mut Health,
	ui_to_update: &mut Vec<UpdatableUiType>,
	audio: &mut Vec<SfxEvent>,
	cfg: GameConfig,
	global_timer: u32,
) -> Result<Option<()>, ()> {
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
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc1 => {
			if stats.armor_points < 200 {
				stats.armor_points = 200;
				stats.is_super_armor = true;

				ui_to_update.push(UpdatableUiType::Armor);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc2 => {
			hp.0 = (hp.0 + 1).min(200);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			ui_to_update.push(UpdatableUiType::Face);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc3 => {
			stats.armor_points = (stats.armor_points + 1).min(200);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Armor);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc4 => {
			inv.cards[Card::BlueCard as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc5 => {
			inv.cards[Card::RedCard as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc6 => {
			inv.cards[Card::YellowCard as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc7 => {
			inv.cards[Card::YellowSkull as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc8 => {
			inv.cards[Card::RedSkull as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc9 => {
			inv.cards[Card::BlueSkull as usize] = true;

			ui_to_update.push(UpdatableUiType::Keys);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSITEMUP"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc10 => {
			if hp.0 < 100 {
				hp.0 = (hp.0 + 10).min(100);

				ui_to_update.push(UpdatableUiType::Hp);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc11 => {
			if hp.0 < 100 {
				hp.0 = (hp.0 + 25).min(100);

				ui_to_update.push(UpdatableUiType::Hp);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc12 => {
			hp.0 = (hp.0 + 100).min(200);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc13 => {
			if hp.0 < 100 {
				hp.0 = 100;
			}
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});
			stats.berserk_timestamp = Some(global_timer);

			Ok(Some(()))
		}
		MobjNum::Misc14 => {
			stats.rad_suit_timestamp = Some(global_timer);

			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});
			Ok(Some(()))
		}
		MobjNum::Misc15 => {
			if !stats.computer_area_map {
				stats.computer_area_map = true;
				stats.item_count += 1;

				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSGETPOW"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc16 => {
			stats.goggles_timestamp = Some(global_timer);
			stats.item_count += 1;

			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Inv => {
			stats.invuln_timestamp = Some(global_timer);
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Face);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Ins => {
			// TODO: make the player's weapon partially invisible
			stats.invis_timestamp = Some(global_timer);
			stats.item_count += 1;

			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Mega => {
			hp.0 = 200;
			stats.armor_points = 200;
			stats.is_super_armor = true;
			stats.item_count += 1;

			ui_to_update.push(UpdatableUiType::Hp);
			ui_to_update.push(UpdatableUiType::Armor);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Clip => {
			let max_ammo = max_ammo[CLIP_IDX] << inv.backpack as u32;

			if inv.ammo[CLIP_IDX] < max_ammo {
				let new_ammo = inv.ammo[CLIP_IDX] + (10 << skill_mult >> from_monster_divisor);
				inv.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc17 => {
			let max_ammo = max_ammo[CLIP_IDX] << inv.backpack as u32;

			if inv.ammo[CLIP_IDX] < max_ammo {
				let new_ammo = inv.ammo[CLIP_IDX] + (50 << skill_mult);
				inv.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc18 => {
			let max_ammo = max_ammo[MISSILE_IDX] << inv.backpack as u32;

			if inv.ammo[MISSILE_IDX] < max_ammo {
				let new_ammo = inv.ammo[MISSILE_IDX] + (1 << skill_mult);
				inv.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc19 => {
			let max_ammo = max_ammo[MISSILE_IDX] << inv.backpack as u32;

			if inv.ammo[MISSILE_IDX] < max_ammo {
				let new_ammo = inv.ammo[MISSILE_IDX] + (5 << skill_mult);
				inv.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc20 => {
			let max_ammo = max_ammo[CELL_IDX] << inv.backpack as u32;

			if inv.ammo[CELL_IDX] < max_ammo {
				let new_ammo = inv.ammo[CELL_IDX] + (20 << skill_mult);
				inv.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc21 => {
			let max_ammo = max_ammo[CELL_IDX] << inv.backpack as u32;

			if inv.ammo[CELL_IDX] < max_ammo {
				let new_ammo = inv.ammo[CELL_IDX] + (100 << skill_mult);
				inv.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc22 => {
			let max_ammo = max_ammo[SHELL_IDX] << inv.backpack as u32;

			if inv.ammo[SHELL_IDX] < max_ammo {
				let new_ammo = inv.ammo[SHELL_IDX] + (4 << skill_mult);
				inv.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc23 => {
			let max_ammo = max_ammo[CLIP_IDX] << inv.backpack as u32;

			if inv.ammo[SHELL_IDX] < max_ammo {
				let new_ammo = inv.ammo[SHELL_IDX] + (20 << skill_mult);
				inv.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});

				Ok(Some(()))
			} else {
				Err(())
			}
		}
		MobjNum::Misc24 => {
			inv.backpack = true;

			let new_clips = inv.ammo[CLIP_IDX] + (10 << skill_mult);
			let new_missiles = inv.ammo[MISSILE_IDX] + (1 << skill_mult);
			let new_cells = inv.ammo[CELL_IDX] + (20 << skill_mult);
			let new_shells = inv.ammo[SHELL_IDX] + (4 << skill_mult);

			inv.ammo[CLIP_IDX] = new_clips.min(max_ammo[CLIP_IDX] << 1);
			inv.ammo[MISSILE_IDX] = new_missiles.min(max_ammo[MISSILE_IDX] << 1);
			inv.ammo[CELL_IDX] = new_cells.min(max_ammo[CELL_IDX] << 1);
			inv.ammo[SHELL_IDX] = new_shells.min(max_ammo[SHELL_IDX] << 1);

			ui_to_update.push(UpdatableUiType::Ammo);
			ui_to_update.push(UpdatableUiType::TotalAmmo);
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSGETPOW"),
				pos: None,
			});

			Ok(Some(()))
		}
		MobjNum::Misc25 => {
			let max_ammo = max_ammo[CELL_IDX] << inv.backpack as u32;
			let mut success = false;

			if !inv.weapon_owned[WeaponType::BFG as usize] {
				inv.weapon_owned[WeaponType::BFG as usize] = true;
				inv.pending_weapon = WeaponType::BFG;

				if cfg.dmatch {
					let new_ammo = inv.ammo[CELL_IDX] + 100;
					inv.ammo[CELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inv.ammo[CELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inv.ammo[CELL_IDX] + (40 << skill_mult);
				inv.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		MobjNum::Misc26 => {
			let mut success = false;

			if !inv.weapon_owned[WeaponType::Chainsaw as usize] {
				inv.weapon_owned[WeaponType::Chainsaw as usize] = true;
				inv.pending_weapon = WeaponType::Chainsaw;

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		MobjNum::Misc27 => {
			let max_ammo = max_ammo[MISSILE_IDX] << inv.backpack as u32;
			let mut success = false;

			if !inv.weapon_owned[WeaponType::Missile as usize] {
				inv.weapon_owned[WeaponType::Missile as usize] = true;
				inv.pending_weapon = WeaponType::Missile;

				if cfg.dmatch {
					let new_ammo = inv.ammo[MISSILE_IDX] + 5;
					inv.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inv.ammo[MISSILE_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inv.ammo[MISSILE_IDX] + (2 << skill_mult);
				inv.ammo[MISSILE_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		MobjNum::Misc28 => {
			let max_ammo = max_ammo[CELL_IDX] << inv.backpack as u32;
			let mut success = false;

			if !inv.weapon_owned[WeaponType::Plasma as usize] {
				inv.weapon_owned[WeaponType::Plasma as usize] = true;
				inv.pending_weapon = WeaponType::Plasma;

				if cfg.dmatch {
					let new_ammo = inv.ammo[CELL_IDX] + 100;
					inv.ammo[CELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inv.ammo[CELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inv.ammo[CELL_IDX] + (40 << skill_mult);
				inv.ammo[CELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		MobjNum::Chaingun => {
			let max_ammo = max_ammo[CLIP_IDX] << inv.backpack as u32;
			let mut success = false;

			if !inv.weapon_owned[WeaponType::Chaingun as usize] {
				inv.weapon_owned[WeaponType::Chaingun as usize] = true;
				inv.pending_weapon = WeaponType::Chaingun;

				if cfg.dmatch {
					let new_ammo = inv.ammo[CLIP_IDX] + 50;
					inv.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inv.ammo[CLIP_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inv.ammo[CLIP_IDX] + (20 << skill_mult >> from_monster_divisor);
				inv.ammo[CLIP_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		MobjNum::Shotgun => {
			let max_ammo = max_ammo[SHELL_IDX] << inv.backpack as u32;
			let mut success = false;

			if !inv.weapon_owned[WeaponType::Shotgun as usize] {
				inv.weapon_owned[WeaponType::Shotgun as usize] = true;
				inv.pending_weapon = WeaponType::Shotgun;

				if cfg.dmatch {
					let new_ammo = inv.ammo[SHELL_IDX] + 20;
					inv.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inv.ammo[SHELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inv.ammo[SHELL_IDX] + (8 << skill_mult >> from_monster_divisor);
				inv.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		MobjNum::Supershotgun => {
			let max_ammo = max_ammo[SHELL_IDX] << inv.backpack as u32;
			let mut success = false;

			if !inv.weapon_owned[WeaponType::SuperShotgun as usize] {
				inv.weapon_owned[WeaponType::SuperShotgun as usize] = true;
				inv.pending_weapon = WeaponType::SuperShotgun;

				if cfg.dmatch {
					let new_ammo = inv.ammo[SHELL_IDX] + 20;
					inv.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

					ui_to_update.push(UpdatableUiType::Ammo);
					ui_to_update.push(UpdatableUiType::TotalAmmo);
				}

				ui_to_update.push(UpdatableUiType::Arms);

				success = true;
			}

			if inv.ammo[SHELL_IDX] != max_ammo && !cfg.dmatch {
				let new_ammo = inv.ammo[SHELL_IDX] + (8 << skill_mult);
				inv.ammo[SHELL_IDX] = new_ammo.min(max_ammo);

				ui_to_update.push(UpdatableUiType::Ammo);
				ui_to_update.push(UpdatableUiType::TotalAmmo);

				success = true;
			}

			if success {
				audio.push(SfxEvent {
					sfx_id: to_u64(b"DSITEMUP"),
					pos: None,
				});
				if cfg.dmatch { Ok(None) } else { Ok(Some(())) }
			} else {
				Err(())
			}
		}
		_ => unreachable!(),
	}
}

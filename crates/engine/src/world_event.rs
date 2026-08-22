use std::error::Error;

use hecs::{Entity, World};

use crate::{NUMCARDS, NUMWEAPONS, PlayerInventory, UpdatableUiType};

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

pub fn execute_events_system(
    world_events: &mut Vec<WorldEvent>, 
    world: &World, 
    player_ent: Entity,
    ui_to_update: &mut Vec<UpdatableUiType>
) -> Result<(), Box<dyn Error>> {
	for event in world_events.drain(..) {
		match event {
			WorldEvent::DamageMobj { target: _, inflictor: _, damage: _ } => {

			},
			WorldEvent::ResetSkullFly { actor_id: _ } => {

			}
			WorldEvent::TouchSpecialThing { special_item: _, picker: _ } => {
                
			}
            WorldEvent::CheatIDKFA => {
                let mut inventory = world.get::<&mut PlayerInventory>(player_ent)?;
                inventory.backpack = true;
                inventory.ammo = [400, 100, 100, 600];
                inventory.weapon_owned = [true; NUMWEAPONS];
                inventory.cards = [true; NUMCARDS];

                ui_to_update.push(UpdatableUiType::Ammo);
                ui_to_update.push(UpdatableUiType::Arms);
                //ui_to_update.push(UpdatableUiType::Face);
                ui_to_update.push(UpdatableUiType::Keys);
                ui_to_update.push(UpdatableUiType::TotalAmmo);

                println!("Very Happy Ammo Added");
            }
            WorldEvent::CheatIDFA => {
                let mut inventory = world.get::<&mut PlayerInventory>(player_ent)?;
                inventory.backpack = true;
                inventory.ammo = [400, 100, 100, 600];
                inventory.weapon_owned = [true; NUMWEAPONS];

                ui_to_update.push(UpdatableUiType::Ammo);
                ui_to_update.push(UpdatableUiType::Arms);
                //ui_to_update.push(UpdatableUiType::Face);
                ui_to_update.push(UpdatableUiType::TotalAmmo);
                
                println!("Ammo (no keys) Added");
            }
            WorldEvent::CheatIDDQD => {

            }
            WorldEvent::CheatNOCLIP => {

            }
		}
	}

    Ok(())
}
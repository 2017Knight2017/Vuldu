use serde::Deserialize;
use strum::IntoEnumIterator;
use toml;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::fs;
use crate::{
    enums::{ActionFunc, MobjFlag, SFX, SpriteNum, StateNum, MobjType}
};

#[derive(Debug, Deserialize)]
pub struct MobjInfo {
    pub doomed_num: u32,
    pub spawn_state: Option<StateNum>,
    pub spawn_health: i32,
    pub see_state: Option<StateNum>,
    pub see_sound: Option<SFX>,
    pub reaction_time: u32,
    pub attack_sound: Option<SFX>,
    pub pain_state: Option<StateNum>,
    pub pain_chance: u8,
    pub pain_sound: Option<SFX>,
    pub melee_state: Option<StateNum>,
    pub missile_state: Option<StateNum>,
    pub death_state: Option<StateNum>,
    pub xdeath_state: Option<StateNum>,
    pub death_sound: Option<SFX>,
    pub speed: f32,
    pub radius: f32,
    pub height: f32,
    pub mass: u32,
    pub damage: u32,
    pub active_sound: Option<SFX>,
    pub flags: Vec<MobjFlag>,
    pub raise_state: Option<StateNum>,
}

#[derive(Debug, Deserialize)]
struct MobjConfig {
    objects: Vec<MobjInfo>,
}

#[derive(Debug, Deserialize)]
pub struct State {
    pub sprite: SpriteNum,
    pub frame: u64,
    pub tics: i64,
    pub action: Option<ActionFunc>,
    pub next_state: Option<StateNum>,
}

#[derive(Debug, Deserialize)]
struct StateConfig {
    states: Vec<State>,
}

#[derive(Debug, Default)]
pub struct Database {
    pub mobjinfo: HashMap<MobjType, MobjInfo>,
    pub states: HashMap<StateNum, State>
}

pub static DB: OnceLock<Database> = OnceLock::new();

pub fn populate_database() -> Result<(), Box<dyn std::error::Error>> {
    let states_content = fs::read_to_string("crates/engine/data_tables/states.toml")?;
    let state_config: StateConfig = toml::from_str(&states_content)?;

    let mobj_content = fs::read_to_string("crates/engine/data_tables/mobjinfo.toml")?;
    let mobj_config: MobjConfig = toml::from_str(&mobj_content)?;

    let state_enum_count = StateNum::iter().count();
    let states_toml_count = state_config.states.len();

    if state_enum_count != states_toml_count {
        return Err(format!(
            "[ERROR]: Length of StateNum ({}) is not equal to the amount of states in states.toml ({})!", 
            state_enum_count, states_toml_count
        ).into());
    }

    let mobjtype_enum_count = MobjType::iter().count();
    let mobjinfo_toml_count = mobj_config.objects.len();

    if mobjtype_enum_count != mobjinfo_toml_count {
        return Err(format!(
            "[ERROR]: Length of MobjType ({}) is not equal to the amount of mobjtypes in mobjinfo.toml ({})!", 
            mobjtype_enum_count, mobjinfo_toml_count
        ).into());
    }

    let mut db = Database::default();

    for (state_num, state) in StateNum::iter().zip(state_config.states) {
        db.states.insert(state_num, state);
    }

    for (mobj_type, mobj_info) in MobjType::iter().zip(mobj_config.objects) {
        db.mobjinfo.insert(mobj_type, mobj_info);
    }

    DB.set(db).unwrap_or_else(|_| eprintln!("Database has already been initialized!"));

    Ok(())
}


use serde::{Deserialize, Deserializer};
use strum::IntoEnumIterator;
use toml;
use rustc_hash::FxHashMap;
use std::sync::OnceLock;
use std::fs;
use crate::{
    enums::{ActionFunc, MobjFlag, SFX, StateNum, MobjType}
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

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct StateRaw {
    #[serde(deserialize_with = "parse_sprite_num")]
    pub sprite: [u8; 4],
    pub frame: u32,
    pub tics: i32,
    pub action: Option<ActionFunc>,
    pub next_state: Option<StateNum>,
}

fn parse_sprite_num<'de, D>(deserializer: D) -> Result<[u8; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    
    if s.len() != 4 {
        return Err(serde::de::Error::custom(format!(
            "Sprite name must be exactly 4 characters long, got: '{}'", s
        )));
    }

    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(s.as_bytes());
    Ok(bytes)
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub sprite: [u8; 4],
    pub frame: u32,
    pub tics: i32,
    pub action: Option<ActionFunc>,
    pub next_state: Option<StateNum>,
    pub cached_rotations: [CachedStateSprite; 9]
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CachedStateSprite {
    pub tex_id: u32,
    pub width: u32,
    pub height: u32,
    pub need_flip: bool,
}

#[derive(Debug, Deserialize)]
struct StateConfig {
    states: Vec<StateRaw>,
}

#[derive(Debug, Default)]
pub struct Database {
    pub mobjinfo: FxHashMap<MobjType, MobjInfo>,
    pub states: FxHashMap<StateNum, State>
}

pub static DB: OnceLock<Database> = OnceLock::new();

pub fn populate_database(texture_data: &FxHashMap<u64, (u32, u32, u32, bool)>) -> Result<(), Box<dyn std::error::Error>> {
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
        let mut cached_rotations = [CachedStateSprite::default(); 9];
        let tex_prefix = state.sprite;
        let frame_letter = (b'A' + state.frame as u8) as char;

        let mut key_0 = pack_sprite_u64(&tex_prefix, frame_letter, 0);
        if !texture_data.contains_key(&key_0) { key_0 = pack_sprite_u64(&tex_prefix, frame_letter, 1); }

        let &(id0, w0, h0, flip0) = texture_data.get(&key_0).unwrap_or(&(0, 64, 64, false));
        cached_rotations[0] = CachedStateSprite { tex_id: id0, width: w0, height: h0, need_flip: flip0 };

        for rot in 1..=8 as usize {
            let lookup_key = pack_sprite_u64(&tex_prefix, frame_letter, rot as u8);
            
            if !texture_data.contains_key(&lookup_key) {
                cached_rotations[rot] = cached_rotations[0];
            } else {
                let &(id, w, h, flip) = texture_data.get(&lookup_key).unwrap_or(&(0, 64, 64, false));
                cached_rotations[rot] = CachedStateSprite { tex_id: id, width: w, height: h, need_flip: flip };
            }
        }
        db.states.insert(state_num, State { 
            sprite: state.sprite, 
            frame: state.frame,
            tics: state.tics,
            action: state.action,
            next_state: state.next_state,
            cached_rotations
        });
    }

    for (mobj_type, mobj_info) in MobjType::iter().zip(mobj_config.objects) {
        db.mobjinfo.insert(mobj_type, mobj_info);
    }

    DB.set(db).unwrap_or_else(|_| eprintln!("Database has already been initialized!"));

    Ok(())
}

pub fn pack_sprite_u64(prefix: &[u8], frame: char, rotation: u8) -> u64 {
    let mut buf = [0u8; 8];

    let p_len = std::cmp::min(prefix.len(), 4);
    buf[..p_len].copy_from_slice(&prefix[..p_len]);

    buf[4] = frame as u8;
    buf[5] = b'0' + rotation;

    u64::from_le_bytes(buf)
}

use serde::{Deserialize, Deserializer};
use strum::IntoEnumIterator;
use toml;
use rustc_hash::FxHashMap;
use wad_parser::TextureId;
use std::{error::Error, sync::OnceLock};
use crate::{ActionFunc, AmmoType, MobjFlagNum, MobjNum, NUMMOBJTYPES, NUMSTATES, NUMWEAPONS, StateNum, WeaponType};

#[derive(Debug, Deserialize)]
pub struct MobjInfo {
    pub doomed_num: u32,
    pub spawn_state: Option<StateNum>,
    pub spawn_health: i32,
    pub see_state: Option<StateNum>,
    #[serde(default, deserialize_with = "parse_sfx_name")]
    pub see_sound: Option<[u8; 8]>,
    pub reaction_time: u32,
    #[serde(default, deserialize_with = "parse_sfx_name")]
    pub attack_sound: Option<[u8; 8]>,
    pub pain_state: Option<StateNum>,
    pub pain_chance: u8,
    #[serde(default, deserialize_with = "parse_sfx_name")]
    pub pain_sound: Option<[u8; 8]>,
    pub melee_state: Option<StateNum>,
    pub missile_state: Option<StateNum>,
    pub death_state: Option<StateNum>,
    pub xdeath_state: Option<StateNum>,
    #[serde(default, deserialize_with = "parse_sfx_name")]
    pub death_sound: Option<[u8; 8]>,
    pub speed: f32,
    pub radius: f32,
    pub height: f32,
    pub mass: u32,
    pub damage: u32,
    #[serde(default, deserialize_with = "parse_sfx_name")]
    pub active_sound: Option<[u8; 8]>,
    pub flags: Vec<MobjFlagNum>,
    pub raise_state: Option<StateNum>,
}

fn parse_sfx_name<'de, D>(deserializer: D) -> Result<Option<[u8; 8]>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    let s = match opt {
        Some(s) => s,
        None => return Ok(None),
    };
    
    if s.len() > 8 {
        return Err(serde::de::Error::custom(format!(
            "SFX name must be 8 characters at most, got: '{}'", s
        )));
    }

    let mut bytes = [0u8; 8];
    bytes[..s.len()].copy_from_slice(s.as_bytes());
    Ok(Some(bytes))
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
    pub tex_id: TextureId,
    pub width: u32,
    pub height: u32,
    pub need_flip: bool,
}

#[derive(Debug, Deserialize)]
struct StateConfig {
    states: Vec<StateRaw>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[allow(dead_code)]
pub struct WeaponInfo {
    ammo: AmmoType,
    up_state: StateNum,
    down_state: StateNum,
    ready_state: StateNum,
    atk_state: StateNum,
    flash_state: Option<StateNum>,
}

#[derive(Debug, Deserialize)]
struct WeaponConfig {
    weapons: Vec<WeaponInfo>,
}

#[derive(Debug, Default)]
pub struct Database {
    pub mobjinfo: FxHashMap<MobjNum, MobjInfo>,
    pub states: FxHashMap<StateNum, State>,
    pub weapon_info: FxHashMap<WeaponType, WeaponInfo>,
}

pub static DB: OnceLock<Database> = OnceLock::new();

pub fn populate_database(texture_data: &FxHashMap<u64, (TextureId, u32, u32, bool)>) -> Result<(), Box<dyn Error>> {
    let states_content = include_str!("../data_tables/states.toml");
    let state_config: StateConfig = toml::from_str(states_content)?;

    let mobj_content = include_str!("../data_tables/mobjinfo.toml");
    let mobj_config: MobjConfig = toml::from_str(mobj_content)?;

    let weapon_content = include_str!("../data_tables/weapons.toml");
    let weapon_config: WeaponConfig = toml::from_str(weapon_content)?;    

    let states_toml_count = state_config.states.len();
    if states_toml_count != NUMSTATES {
        return Err(format!(
            "[ERROR]: Length of StateNum ({}) is not equal to the amount of states in states.toml ({})!", 
            NUMSTATES, states_toml_count
        ).into());
    }

    let mobjinfo_toml_count = mobj_config.objects.len();
    if mobjinfo_toml_count != NUMMOBJTYPES {
        return Err(format!(
            "[ERROR]: Length of MobjNum ({}) is not equal to the amount of mobjnums in mobjinfo.toml ({})!", 
            NUMMOBJTYPES, mobjinfo_toml_count
        ).into());
    };

    let weapon_toml_count = weapon_config.weapons.len();
    if weapon_toml_count != NUMWEAPONS {
        return Err(format!(
            "[ERROR]: Length of WeaponType ({}) is not equal to the amount of weapons in weapons.toml ({})!", 
            NUMWEAPONS, weapon_toml_count
        ).into());
    };

    let mut db = Database::default();

    for (state_num, state) in StateNum::iter().zip(state_config.states) {
        let mut cached_rotations = [CachedStateSprite::default(); 9];
        let tex_prefix = state.sprite;
        let frame_letter = (b'A' + state.frame as u8) as char;

        let mut key_0 = pack_sprite_u64(&tex_prefix, frame_letter, 0);
        if !texture_data.contains_key(&key_0) { key_0 = pack_sprite_u64(&tex_prefix, frame_letter, 1); }

        let &(tex_id, width, height, need_flip) = texture_data.get(&key_0).unwrap_or(&(TextureId(0), 64, 64, false));
        cached_rotations[0] = CachedStateSprite { tex_id, width, height, need_flip };

        for rot in 1..=8_usize {
            let lookup_key = pack_sprite_u64(&tex_prefix, frame_letter, rot as u8);
            
            if !texture_data.contains_key(&lookup_key) {
                cached_rotations[rot] = cached_rotations[0];
            } else {
                let &(tex_id, width, height, need_flip) = texture_data.get(&lookup_key).unwrap_or(&(TextureId(0), 64, 64, false));
                cached_rotations[rot] = CachedStateSprite { tex_id, width, height, need_flip };
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

    for (mobj_type, mobj_info) in MobjNum::iter().zip(mobj_config.objects) {
        db.mobjinfo.insert(mobj_type, mobj_info);
    }

    for (weapon_type, weapon_info) in WeaponType::iter().zip(weapon_config.weapons) {
        db.weapon_info.insert(weapon_type, weapon_info);
    }

    DB.set(db).unwrap_or_else(|_| eprintln!("Database has already been initialized!"));

    Ok(())
}

pub fn pack_sprite_u64(prefix: &[u8], frame: char, rotation: u8) -> u64 {
    let mut buf = [0u8; 8];

    let p_len = usize::min(prefix.len(), 4);
    buf[..p_len].copy_from_slice(&prefix[..p_len]);

    buf[4] = frame as u8;
    buf[5] = b'0' + rotation;

    u64::from_le_bytes(buf)
}

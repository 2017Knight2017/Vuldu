use crate::{ActionFunc, AmmoType, MobjFlagNum, NUMMOBJTYPES, NUMSTATES, NUMWEAPONS, StateNum};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Deserializer};
use std::sync::OnceLock;
use wad_parser::TextureId;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct MobjInfo {
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
			"SFX name must be 8 characters at most, got: '{}'",
			s
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
struct StateRaw {
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
			"Sprite name must be exactly 4 characters long, got: '{}'",
			s
		)));
	}

	let mut bytes = [0u8; 4];
	bytes.copy_from_slice(s.as_bytes());
	Ok(bytes)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct State {
	#[allow(dead_code)]
	pub sprite: [u8; 4],
	pub frame: u32,
	pub tics: i32,
	pub action: Option<ActionFunc>,
	pub next_state: Option<StateNum>,
	pub cached_rotations: [CachedStateSprite; 9],
}

#[derive(Debug, Clone, Copy, Default)]
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
pub(crate) struct WeaponInfo {
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
pub(crate) struct Database {
	pub mobjinfo: Vec<MobjInfo>,
	pub states: Vec<State>,
	#[allow(dead_code)]
	pub weapon_info: Vec<WeaponInfo>,
}

pub(crate) static DB: OnceLock<Database> = OnceLock::new();

pub fn populate_database(
	texture_data: &FxHashMap<u64, (TextureId, u32, u32, bool)>,
) -> Result<(), toml::de::Error> {
	let states_content = include_str!("../data_tables/states.toml");
	let state_config: StateConfig = toml::from_str(states_content)?;

	let mobj_content = include_str!("../data_tables/mobjinfo.toml");
	let mobj_config: MobjConfig = toml::from_str(mobj_content)?;

	let weapon_content = include_str!("../data_tables/weapons.toml");
	let weapon_config: WeaponConfig = toml::from_str(weapon_content)?;

	debug_assert_eq!(state_config.states.len(), NUMSTATES);
	debug_assert_eq!(mobj_config.objects.len(), NUMMOBJTYPES);
	debug_assert_eq!(weapon_config.weapons.len(), NUMWEAPONS);

	let states: Vec<State> = state_config
		.states
		.into_iter()
		.map(|state| {
			let mut cached_rotations = [CachedStateSprite::default(); 9];
			let tex_prefix = state.sprite;
			let frame_letter = (b'A' + state.frame as u8) as char;

			let mut key_0 = pack_sprite_u64(&tex_prefix, frame_letter, 0);
			if !texture_data.contains_key(&key_0) {
				key_0 = pack_sprite_u64(&tex_prefix, frame_letter, 1);
			}

			let &(tex_id, width, height, need_flip) =
				texture_data
					.get(&key_0)
					.unwrap_or(&(TextureId(0), 64, 64, false));
			cached_rotations[0] = CachedStateSprite {
				tex_id,
				width,
				height,
				need_flip,
			};

			for rot in 1..=8_usize {
				let lookup_key = pack_sprite_u64(&tex_prefix, frame_letter, rot as u8);

				if !texture_data.contains_key(&lookup_key) {
					cached_rotations[rot] = cached_rotations[0];
				} else {
					let &(tex_id, width, height, need_flip) = texture_data
						.get(&lookup_key)
						.unwrap_or(&(TextureId(0), 64, 64, false));
					cached_rotations[rot] = CachedStateSprite {
						tex_id,
						width,
						height,
						need_flip,
					};
				}
			}

			State {
				sprite: state.sprite,
				frame: state.frame,
				tics: state.tics,
				action: state.action,
				next_state: state.next_state,
				cached_rotations,
			}
		})
		.collect();

	let _ = DB.set(Database {
		states,
		mobjinfo: mobj_config.objects,
		weapon_info: weapon_config.weapons,
	});

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

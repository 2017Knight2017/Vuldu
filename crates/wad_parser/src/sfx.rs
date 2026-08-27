use rustc_hash::FxHashMap;

use crate::{WadManager, to_u64};

pub const SFX_NAMES: [&[u8]; 108] = [
	b"DSPISTOL",
	b"DSSHOTGN",
	b"DSSGCOCK",
	b"DSDSHTGN",
	b"DSDBOPN",
	b"DSDBCLS",
	b"DSDBLOAD",
	b"DSPLASMA",
	b"DSBFG",
	b"DSSAWUP",
	b"DSSAWIDL",
	b"DSSAWFUL",
	b"DSSAWHIT",
	b"DSRLAUNC",
	b"DSRXPLOD",
	b"DSFIRSHT",
	b"DSFIRXPL",
	b"DSPSTART",
	b"DSPSTOP",
	b"DSDOROPN",
	b"DSDORCLS",
	b"DSSTNMOV",
	b"DSSWTCHN",
	b"DSSWTCHX",
	b"DSPLPAIN",
	b"DSDMPAIN",
	b"DSPOPAIN",
	b"DSVIPAIN",
	b"DSMNPAIN",
	b"DSPEPAIN",
	b"DSSLOP",
	b"DSITEMUP",
	b"DSWPNUP",
	b"DSOOF",
	b"DSTELEPT",
	b"DSPOSIT1",
	b"DSPOSIT2",
	b"DSPOSIT3",
	b"DSBGSIT1",
	b"DSBGSIT2",
	b"DSSGTSIT",
	b"DSCACSIT",
	b"DSBRSSIT",
	b"DSCYBSIT",
	b"DSSPISIT",
	b"DSBSPSIT",
	b"DSKNTSIT",
	b"DSVILSIT",
	b"DSMANSIT",
	b"DSPESIT",
	b"DSSKLATK",
	b"DSSGTATK",
	b"DSSKEPCH",
	b"DSVILATK",
	b"DSCLAW",
	b"DSSKESWG",
	b"DSPLDETH",
	b"DSPDIEHI",
	b"DSPODTH1",
	b"DSPODTH2",
	b"DSPODTH3",
	b"DSBGDTH1",
	b"DSBGDTH2",
	b"DSSGTDTH",
	b"DSCACDTH",
	b"DSSKLDTH",
	b"DSBRSDTH",
	b"DSCYBDTH",
	b"DSSPIDTH",
	b"DSBSPDTH",
	b"DSVILDTH",
	b"DSKNTDTH",
	b"DSPEDTH",
	b"DSSKEDTH",
	b"DSPOSACT",
	b"DSBGACT",
	b"DSDMACT",
	b"DSBSPACT",
	b"DSBSPWLK",
	b"DSVILACT",
	b"DSNOWAY",
	b"DSBAREXP",
	b"DSPUNCH",
	b"DSHOOF",
	b"DSMETAL",
	b"DSCHGUN",
	b"DSTINK",
	b"DSBDOPN",
	b"DSBDCLS",
	b"DSITMBK",
	b"DSFLAME",
	b"DSFLAMST",
	b"DSGETPOW",
	b"DSBOSPIT",
	b"DSBOSCUB",
	b"DSBOSSIT",
	b"DSBOSPN",
	b"DSBOSDTH",
	b"DSMANATK",
	b"DSMANDTH",
	b"DSSSSIT",
	b"DSSSDTH",
	b"DSKEENPN",
	b"DSKEENDT",
	b"DSSKEACT",
	b"DSSKESIT",
	b"DSSKEATK",
	b"DSRADIO",
];

pub struct DoomSfx {
	pub sample_rate: u32,
	pub samples: Vec<f32>,
}

impl DoomSfx {
	pub fn parse(raw_data: &[u8]) -> Result<Self, String> {
		if raw_data.len() < 8 {
			return Err("Sound lump too short".to_string());
		}

		let magic = u16::from_le_bytes(raw_data[0..2].try_into().unwrap());
		if magic != 3 {
			return Err("Invalid Doom sound header".to_string());
		}

		let sample_rate = u16::from_le_bytes(raw_data[2..4].try_into().unwrap()) as u32;
		let num_samples = u32::from_le_bytes(raw_data[4..8].try_into().unwrap()) as usize;

		let pcm_data = &raw_data[8..];
		let actual_samples = pcm_data.len().min(num_samples);

		let samples: Vec<f32> = pcm_data[..actual_samples]
			.iter()
			.map(|&byte| (byte as f32 - 128.0) / 128.0)
			.collect();

		Ok(Self {
			sample_rate,
			samples,
		})
	}
}

impl WadManager {
	pub fn bake_sfx(&self) -> FxHashMap<u64, DoomSfx> {
		let mut sound_cache: FxHashMap<u64, DoomSfx> = FxHashMap::default();

		for sound_name in SFX_NAMES {
			if let Ok(raw_bytes) = self.get_data(sound_name)
				&& let Ok(doom_sound) = DoomSfx::parse(raw_bytes)
			{
				sound_cache.insert(to_u64(sound_name), doom_sound);
			}
		}

		sound_cache
	}
}

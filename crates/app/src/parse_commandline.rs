use clap::Parser;
use std::path::PathBuf;

const SKILL_HELP: &str = "Difficulty, from I'm Too Young To Die (0) to Nightmare! (4) inclusively.";
const WIREFRAME_HELP: &str = "Switch the renderer to the Wireframe mode, with no wall textures shown. It's useful when, for example, you want to check how well level surfaces got triangulated.";
const BYTE_SHADOWS_HELP: &str = "Make shadows submit values of light in 0..255 range (default range is 0..32).";
const FAST_MONSTERS_HELP: &str = "Currently does nothing.";
const COOP_HELP: &str = "Turn on the Co-op mode. Currently just spawns additional objects on a level.";
const DEATHMATCH_HELP: &str = "Turn on the Deathmatch mode. Currently just prevents keys from spawning on a level.";
const NO_MONSTERS_HELP: &str = "Prevent monsters from spawning.";

#[derive(Parser, Debug)]
#[command(name = "Vuldu")]
#[command(author = "2017Knight2017")]
#[command(version = "0.1.0")]
#[command(about = "Vulkan-powered Doom port written in Rust")]
pub struct Args {
	#[arg(short, long, value_name = "FILE")]	
	pub iwad: PathBuf,

	#[arg(short, long, value_name = "FILE")]
    pub pwads: Vec<PathBuf>,

    #[arg(short, long, default_value = "1", value_name = "MAP_NUM")]
    pub map: u8,

	#[arg(short, long, help = WIREFRAME_HELP)]
	pub wireframe: bool,

	#[arg(short, long = "byte-shadows", help = BYTE_SHADOWS_HELP)]
	pub byte_shadows: bool,

	#[arg(short, long = "skill", default_value = "2", value_name = "SKILL_NUM", help = SKILL_HELP)]
	pub skill_level: u8,

	#[arg(short, long = "fast-monsters", help = FAST_MONSTERS_HELP)]
	pub fast_monsters: bool,

	#[arg(short, long, help = COOP_HELP)]
	pub coop: bool,

	#[arg(short, long, help = DEATHMATCH_HELP)]
	pub deathmatch: bool,

	#[arg(short, long, help = NO_MONSTERS_HELP)]
	pub no_monsters: bool,
}

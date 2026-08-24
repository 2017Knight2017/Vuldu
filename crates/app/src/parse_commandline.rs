use clap::Parser;
use std::path::PathBuf;

const SKILL_HELP: &'static str = "From I'm Too Young To Die (0) to Nightmare! (4).";

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

	#[arg(long)]
	pub wireframe: bool,

	#[arg(long = "byte-shadows")]
	pub byte_shadows: bool,

	#[arg(short, long = "skill", default_value = "2", value_name = "SKILL_NUM", help = SKILL_HELP)]
	pub skill_level: u8,

	#[arg(short, long = "fast-monsters")]
	pub fast_monsters: bool
}

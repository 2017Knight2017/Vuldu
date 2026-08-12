use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "Vuldu")]
#[command(author = "2017Knight2017")]
#[command(version = "0.1.0")]
#[command(about = "Vulkan-powered Doom port written in Rust", long_about = None)]
pub struct Args {
	#[arg(short, long, value_name = "FILE")]	
	pub iwad: PathBuf,

	#[arg(short = 'w', long = "wad", value_name = "FILE")]
    pub pwads: Vec<PathBuf>,

    #[arg(short, long, default_value = "1", value_name = "MAP_NUM")]
    pub map: u8
}

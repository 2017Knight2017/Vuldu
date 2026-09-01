use crate::SectorId;

#[derive(Debug)]
pub struct ActiveEffect {
	pub sector: SectorId,
	pub kind: EffectKind,
}

#[derive(Debug)]
pub enum EffectKind {
	Door(Door),
	Plat(Plat),
	Floor(PlaneMove),
	Ceiling(PlaneMove),
	Light(Light),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorMoveDir {
	Up,
	Wait,
	Down,
}

#[derive(Debug)]
pub struct Door {
	pub top_h: f32,
	pub speed: f32,
	pub dir: SectorMoveDir,
	pub top_wait: u32,
	pub countdown: u32,
	pub blazing: bool,
}

#[derive(Debug)]
pub struct Plat {
	pub low: f32,
	pub high: f32,
	pub speed: f32,
	pub wait: u32,
	pub count: u32,
	pub status: PlatStatus,
	pub old_status: PlatStatus,
	pub crush: bool,
	pub tag: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatStatus {
	Up,
	Down,
	Waiting,
	Stasis,
}

#[derive(Debug)]
pub struct PlaneMove {
	pub dest_h: f32,
	pub speed: f32,
	pub dir: SectorMoveDir,
	pub crush: bool,
	//pub on_finish: FinishAction,
}

#[derive(Debug)]
pub enum Light {
	Flash {
		bright: u16,
		dark: u16,
		count: u32,
		max_time: u32,
		min_time: u32,
	},
	Strobe {
		bright: u16,
		dark: u16,
		count: u32,
		bright_time: u32,
		dark_time: u32,
	},
	Glow {
		min: u16,
		max: u16,
		dir: i8,
	},
	FireFlicker {
		bright: u16,
		dark: u16,
		count: u32,
	},
}

use hecs::Entity;
use crate::{ActionFunc, CachedStateSprite, Direction, MobjFlags, MobjNum, StateNum};

macro_rules! define_markers {
    ($($name:ident);* $(;)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $name;
        )*
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub prev_x: f32,
	pub prev_y: f32,
	pub prev_z: f32,
}

pub struct InstantMoveIntent {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub new_sector: Option<usize>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerRotation {
	pub angle: u32,
    pub prev_angle: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterRotation {
    // move_dir's range = 0..=7
	pub move_dir: Option<Direction>,
    pub move_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReactionTime(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct SpriteAnimation {
    pub cached_rotations: [CachedStateSprite; 9],
    pub top_offset_shift: i16
}

pub struct MobjAi {
    pub current_state: StateNum,
    pub tics_left: i32,
    pub action: Option<ActionFunc>,
    pub threshold: u32,
    pub reaction_time: u32
}

define_markers! {
    PlayerMarker;
    Special;
    Solid;
    Shootable;
    NoSector;
    NoBlockmap;
    Ambush;
    JustHit;
    JustAttacked;
    SpawnCeiling;
    NoGravity;
    DropOff;
    Pickup;
    NoClip;
    Slide;
    Float;
    Teleport;
    Missile;
    Dropped;
    Shadow;
    NoBlood;
    InFloat;
    CountKill;
    CountItem;
    SkullFly;
    NotDMatch;

    Active;
    Idle;
    Corpse;

    PlayerShoot;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponOverlay {
    pub state_idx: u32,
    pub tics: i32,
    pub sx: f32,
    pub sy: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentSector(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobjType {
    pub type_: MobjNum,
    pub flags: MobjFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target(pub Entity);

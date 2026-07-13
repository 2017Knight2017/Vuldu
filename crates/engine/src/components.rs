use crate::{
    enums::{SFX, StateNum},
    data_tables::CachedStateSprite
};
use hecs::Bundle;

macro_rules! define_markers {
    ($($name:ident);* $(;)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $name;
        )*
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub prev_x: f32,
	pub prev_y: f32,
	pub prev_z: f32,
	pub angle: u32,
    pub prev_angle: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Physics {
    pub speed: f32,
    pub mass: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub radius: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PainReaction {
    pub chance: u8,
    pub sound: Option<SFX>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiAgent {
    pub reaction_time: u32,
    pub active_sound: Option<SFX>,
    pub see_sound: Option<SFX>,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteAnimation {
    pub current_state: Option<StateNum>, 
    pub tics_left: i32,
    pub cached_rotations: [CachedStateSprite; 9],
    pub top_offset_shift: i16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterBrainConfig {
    pub spawn_state: Option<StateNum>,
    pub see_state: Option<StateNum>,
    pub missile_state: Option<StateNum>,
    pub pain_state: Option<StateNum>,
    pub death_state: Option<StateNum>,
    pub xdeath_state: Option<StateNum>,
    pub raise_state: Option<StateNum>,
    pub death_sound: Option<SFX>,
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
    Corpse;
    InFloat;
    CountKill;
    CountItem;
    SkullFly;
    NotDMatch;
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

#[derive(Bundle)]
pub struct MobjBundle {
    pub transform: Transform,
    pub velocity: Velocity,
    pub bbox: BoundingBox,
    pub sprite_anim: SpriteAnimation,
}

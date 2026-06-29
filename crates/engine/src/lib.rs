use hecs;

const TICKRATE: u32 = 35;
const NUMAMMO: usize = 4;
const NUMPOWERS: usize = 6;
const NUMWEAPONS: usize = 9;
const NUMCARDS: usize = 6;
const NUMSPRITES: usize = 138;

#[derive(Clone, Copy, Default)]
pub struct PlayerInput {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub shoot: bool,
    pub mouse_delta_x: f32,
}

pub enum Sprite {
    TROO,SHTG,PUNG,PISG,PISF,SHTF,SHT2,CHGG,CHGF,MISG,
	MISF,SAWG,PLSG,PLSF,BFGG,BFGF,BLUD,PUFF,BAL1,BAL2,
    PLSS,PLSE,MISL,BFS1,BFE1,BFE2,TFOG,IFOG,PLAY,POSS,
	SPOS,VILE,FIRE,FATB,FBXP,SKEL,MANF,FATT,CPOS,SARG,
    HEAD,BAL7,BOSS,BOS2,SKUL,SPID,BSPI,APLS,APBX,CYBR,
    PAIN,SSWV,KEEN,BBRN,BOSF,ARM1,ARM2,BAR1,BEXP,FCAN,
    BON1,BON2,BKEY,RKEY,YKEY,BSKU,RSKU,YSKU,STIM,MEDI,
    SOUL,PINV,PSTR,PINS,MEGA,SUIT,PMAP,PVIS,CLIP,AMMO,
	ROCK,BROK,CELL,CELP,SHEL,SBOX,BPAK,BFUG,MGUN,CSAW,
    LAUN,PLAS,SHOT,SGN2,COLU,SMT2,GOR1,POL2,POL5,POL4,
    POL3,POL1,POL6,GOR2,GOR3,GOR4,GOR5,SMIT,COL1,COL2,
    COL3,COL4,CAND,CBRA,COL6,TRE1,TRE2,ELEC,CEYE,FSKU,
    COL5,TBLU,TGRN,TRED,SMBT,SMGT,SMRT,HDB1,HDB2,HDB3,
    HDB4,HDB5,HDB6,POB1,POB2,BRS1,TLMP,TLP2
}

pub enum ActionFunction {
	Light0,
	WeaponReady,
	Lower,
	Raise,
	Punch,
	ReFire,
	FirePistol,
	Light1,
	FireShotgun,
	Light2,
	FireShotgun2,
	CheckReload,
	OpenShotgun2,
	LoadShotgun2,
	CloseShotgun2,
	FireCGun,
	GunFlash,
	FireMissile,
	Saw,
	FirePlasma,
	BFGSound,
	FireBFG,
	BFGSpray,
	Explode,
	Pain,
	PlayerScream,
	Fall,
	XScream,
	Look,
	Chase,
	FaceTarget,
	PosAttack,
	Scream,
	SPosAttack,
	VileChase,
	VileStart,
	VileTarget,
	VileAttack,
	StartFire,
	Fire,
	FireCrackle,
	Tracer,
	SkelWhoosh,
	SkelFist,
	SkelMissile,
	FatRaise,
	FatAttack1,
	FatAttack2,
	FatAttack3,
	BossDeath,
	CPosAttack,
	CPosRefire,
	TroopAttack,
	SargAttack,
	HeadAttack,
	BruisAttack,
	SkullAttack,
	Metal,
	SpidRefire,
	BabyMetal,
	BspiAttack,
	Hoof,
	CyberAttack,
	PainAttack,
	PainDie,
	KeenDie,
	BrainPain,
	BrainScream,
	BrainDie,
	BrainAwake,
	BrainSpit,
	SpawnSound,
	SpawnFly,
	BrainExplode,
}

pub enum MobjType {
    Player,
    Possessed,
    Shotguy,
    Vile,
    Fire,
    Undead,
    Tracer,
    Smoke,
    Fatso,
    Fatshot,
    Chainguy,
    Troop,
    Sergeant,
    Shadows,
    Head,
    Bruiser,
    Bruisershot,
    Knight,
    Skull,
    Spider,
    Baby,
    Cyborg,
    Pain,
    Wolfss,
    Keen,
    Bossbrain,
    Bossspit,
    Bosstarget,
    Spawnshot,
    Spawnfire,
    Barrel,
    Troopshot,
    Headshot,
    Rocket,
    Plasma,
    BFG,
    Arachplaz,
    Puff,
    Blood,
    Tfog,
    Ifog,
    Teleportman,
    ExtraBFG,
    Misc0,
    Misc1,
    Misc2,
    Misc3,
    Misc4,
    Misc5,
    Misc6,
    Misc7,
    Misc8,
    Misc9,
    Misc10,
    Misc11,
    Misc12,
    Inv,
    Misc13,
    Ins,
    Misc14,
    Misc15,
    Misc16,
    Mega,
    Clip,
    Misc17,
    Misc18,
    Misc19,
    Misc20,
    Misc21,
    Misc22,
    Misc23,
    Misc24,
    Misc25,
    Chaingun,
    Misc26,
    Misc27,
    Misc28,
    Shotgun,
    Supershotgun,
    Misc29,
    Misc30,
    Misc31,
    Misc32,
    Misc33,
    Misc34,
    Misc35,
    Misc36,
    Misc37,
    Misc38,
    Misc39,
    Misc40,
    Misc41,
    Misc42,
    Misc43,
    Misc44,
    Misc45,
    Misc46,
    Misc47,
    Misc48,
    Misc49,
    Misc50,
    Misc51,
    Misc52,
    Misc53,
    Misc54,
    Misc55,
    Misc56,
    Misc57,
    Misc58,
    Misc59,
    Misc60,
    Misc61,
    Misc62,
    Misc63,
    Misc64,
    Misc65,
    Misc66,
    Misc67,
    Misc68,
    Misc69,
    Misc70,
    Misc71,
    Misc72,
    Misc73,
    Misc74,
    Misc75,
    Misc76,
    Misc77,
    Misc78,
    Misc79,
    Misc80,
    Misc81,
    Misc82,
    Misc83,
    Misc84,
    Misc85,
    Misc86
}

pub enum PlayerState
{
    PSTLive,
    PSTDead,
    PSTReborn		
}

pub struct Transform {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub angle: u32
}

pub struct Velocity {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

pub struct Health(pub i32);

pub struct ActorState {
    pub mobj_type: MobjType,
    pub current_state_idx: usize,
    pub tics: i32,
    pub flags: i32,
}

pub struct Speed {
	pub default: u32,
	pub nightmare: Option<u32>
}

pub struct Damage {
	pub melee: Option<u32>,
	pub far: Option<u32>,
	pub nightmare: Option<u32>
}

pub struct BoundingBox {
    pub radius: f32,
    pub height: f32,
}

pub struct PhysicsEnvironment {
    pub floor_z: f32,
    pub ceiling_z: f32,
}

pub struct PlayerMarker;

pub struct PlayerCamera {
    pub view_z: f32,
    pub view_height: f32,
    pub delta_view_height: f32,
    pub bob: f32,
}

pub struct PlayerStats {
    pub state: PlayerState,
    pub armor_points: i32,
    pub armor_type: i32,
    pub kill_count: i32,
    pub item_count: i32,
    pub secret_count: i32,
}

pub struct PlayerInventory {
    pub ready_weapon: u32,
    pub pending_weapon: u32,
    pub backpack: bool,
    pub cards: [bool; NUMCARDS],
    pub weapon_owned: [bool; NUMWEAPONS],
	pub ammo: [i32; NUMAMMO],
    pub max_ammo: [i32; NUMAMMO],
}

pub struct WeaponOverlay {
    pub state_idx: u32,
    pub tics: i32,
    pub sx: f32,
    pub sy: f32,
}

pub fn update_physics(world: &mut hecs::World, input: &PlayerInput) {
    for (vel, _pos, _player) in world.query_mut::<(&mut Velocity, &mut Transform, &PlayerMarker)>() {
        
        let mut move_vec_x = 0.0;
        let mut move_vec_z = 0.0;

        if input.move_forward  { move_vec_z -= 1.0; }
        if input.move_backward { move_vec_z += 1.0; }
        if input.move_left     { move_vec_x -= 1.0; }
        if input.move_right    { move_vec_x += 1.0; }

        let speed = 8.0;
        vel.x = move_vec_x * speed;
        vel.z = move_vec_z * speed;
    }

    for (pos, vel) in world.query_mut::<(&mut Transform, &Velocity)>() {
        pos.x += vel.x;
        pos.z += vel.z;
    }
}

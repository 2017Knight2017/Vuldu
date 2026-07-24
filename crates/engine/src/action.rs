use hecs::QueryBorrow;
use serde::Deserialize;
use wad_parser::to_u64;
use crate::{ANG45, Active, DB, MobjType, MonsterRotation, PlayerMarker, Position, Random, SfxEvent, SpriteAnimation, point_to_angle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ActionFunc {
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

pub fn chase_system(
	mut query: QueryBorrow<'_, (&mut MonsterRotation, &Position, &MobjType, &SpriteAnimation, &Active)>, 
	mut player_query: QueryBorrow<'_, (&Position, &PlayerMarker)>,
	random: &mut Random,
	audio_buffer: &mut Vec<SfxEvent>
) {
	for (rot, pos, mobj_type, sprite_anim, _active) in query.iter() {
		let (player_pos, _) = player_query.iter().next().unwrap();
		let dx = player_pos.x - pos.x;
		let dy = player_pos.z - pos.z;

		let angle = point_to_angle(-dx, dy);
		rot.move_dir = ((angle.wrapping_add(ANG45/2)) >> 29) & 0b111;

		let db = DB.get().expect("DB has not been initialized!");
		if let Some(active_sound) = db.mobjinfo[&mobj_type.0].active_sound {
			if random.p() & 0xFF < 2 && sprite_anim.tics_left == 1 {
				audio_buffer.push(SfxEvent { sfx_id: to_u64(&active_sound), position: Some((pos.x, pos.y, pos.z)) })
			}
		}
	}
}

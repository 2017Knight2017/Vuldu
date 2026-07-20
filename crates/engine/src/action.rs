use hecs::QueryBorrow;
use serde::Deserialize;
use crate::{ANG45, Active, MonsterRotation, PlayerMarker, Position, point_to_angle};

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
	mut query: QueryBorrow<'_, (&mut MonsterRotation, &Position, &Active)>, 
	mut player_query: QueryBorrow<'_, (&Position, &PlayerMarker)>
) {
	for (rot, pos, _active) in query.iter() {
		let (player_pos, _) = player_query.iter().next().unwrap();
		let dx = player_pos.x - pos.x;
		let dy = player_pos.z - pos.z;

		let angle = point_to_angle(-dx, dy);
		rot.move_dir = ((angle.wrapping_add(ANG45/2)) >> 29) & 0b111;
	}
}

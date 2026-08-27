use crate::{
	CurrentSector, DB, Health, Idle, InstantMoveIntent, MobjAi, MobjFlagCommand, MobjFlags,
	MobjType, MonsterRotation, PLAYERHEIGHT, PlayerMarker, Position, Random, SfxEvent, SkillLevel,
	SpriteAnimation, StateNum, Target, Traversal, WorldEvent, in_fov, p_check_melee_range,
	p_check_missile_range, p_check_sight, p_move, p_new_chase_dir, wake_up_monster,
};
use hecs::{CommandBuffer, Entity, World};
use serde::Deserialize;
use wad_parser::{Level, to_u64};

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

/// Must be called before animation_system
pub fn ai_system(world: &World, action_buffer: &mut Vec<(Entity, ActionFunc)>) {
	let mut query = world.query::<(Entity, &mut MobjAi, &mut SpriteAnimation)>();
	for (ent, ai, anim) in query.iter() {
		if ai.tics_left <= 0 {
			continue;
		}

		ai.tics_left -= 1;
		if ai.tics_left == 0 {
			let db = DB.get().unwrap();

			let current_state = db.states[&ai.current_state];

			if let Some(next_state_num) = current_state.next_state {
				set_mobj_state(ent, ai, anim, next_state_num, action_buffer, 0);
			}
		}
	}
}

pub fn set_mobj_state(
	ent: Entity,
	ai: &mut MobjAi,
	anim: &mut SpriteAnimation,
	state_num: StateNum,
	action_buffer: &mut Vec<(Entity, ActionFunc)>,
	tics_to_add: i32,
) {
	let db = DB.get().unwrap();

	ai.current_state = state_num;

	let state = db.states[&state_num];
	ai.tics_left = state.tics + tics_to_add;
	anim.cached_rotations = state.cached_rotations;

	if let Some(action) = state.action {
		action_buffer.push((ent, action));
	}
}

#[allow(clippy::too_many_arguments)]
pub fn action_system(
	world: &World,
	action_buffer: &mut Vec<(Entity, ActionFunc)>,
	random: &mut Random,
	level: &Level,
	game_skill: SkillLevel,
	fast_monsters: bool,
	audio_buffer: &mut Vec<SfxEvent>,
	blocklists: &[Vec<Entity>],
	world_events: &mut Vec<WorldEvent>,
	mobj_flag_buffer: &mut Vec<MobjFlagCommand>,
	traversal: &mut Traversal,
) {
	let mut local_queue = Vec::new();

	while !action_buffer.is_empty() {
		std::mem::swap(action_buffer, &mut local_queue);

		for (ent, action) in local_queue.drain(..) {
			#[allow(clippy::single_match)]
			match action {
				ActionFunc::Chase => chase(
					world,
					ent,
					random,
					level,
					game_skill,
					fast_monsters,
					audio_buffer,
					blocklists,
					world_events,
					mobj_flag_buffer,
					action_buffer,
					traversal,
				),
				_ => {}
			}
		}
	}
}

/// Must be called after propagate_sound_system
#[allow(clippy::too_many_arguments)]
pub fn check_sound_system(
	world: &World,
	level: &mut Level,
	random: &mut Random,
	command_buffer: &mut CommandBuffer,
	sound_targets: &mut [Option<Entity>],
	traversal: &mut Traversal,
	audio_buffer: &mut Vec<SfxEvent>,
	action_buffer: &mut Vec<(Entity, ActionFunc)>,
) {
	let db = DB.get().unwrap();

	let mut query = world
		.query::<(
			Entity,
			&mut SpriteAnimation,
			&mut MobjAi,
			&Position,
			&CurrentSector,
			&MobjType,
		)>()
		.with::<&Idle>();
	for (entity, anim, ai, pos, current_sector, mobj_type) in query.iter() {
		if let Some(sound_target_entity) = sound_targets[current_sector.0.0] {
			if mobj_type.flags.contains(MobjFlags::AMBUSH) {
				let Ok(target_pos) = world.get::<&Position>(sound_target_entity) else {
					continue;
				};
				let Ok(target_sector) = world.get::<&CurrentSector>(sound_target_entity) else {
					continue;
				};
				let Ok(target_type) = world.get::<&MobjType>(sound_target_entity) else {
					continue;
				};

				if !p_check_sight(
					pos,
					current_sector,
					db.mobjinfo[&mobj_type.type_].height,
					&target_pos,
					&target_sector,
					db.mobjinfo[&target_type.type_].height,
					level,
					traversal,
				) {
					continue;
				}
			}

			wake_up_monster(
				entity,
				pos,
				mobj_type,
				sound_target_entity,
				anim,
				ai,
				random,
				command_buffer,
				audio_buffer,
				action_buffer,
			);
		}
	}
}

pub fn check_sight_system(
	world: &World,
	level: &Level,
	traversal: &mut Traversal,
	random: &mut Random,
	command_buffer: &mut CommandBuffer,
	audio_buffer: &mut Vec<SfxEvent>,
	action_buffer: &mut Vec<(Entity, ActionFunc)>,
) {
	let db = DB.get().unwrap();

	let mut query = world
		.query::<(
			Entity,
			&mut SpriteAnimation,
			&mut MobjAi,
			&Position,
			&CurrentSector,
			&MonsterRotation,
			&MobjType,
		)>()
		.with::<&Idle>();
	let mut players_query = world
		.query::<(Entity, &Position, &CurrentSector, &MobjType)>()
		.with::<&PlayerMarker>();

	for (entity, anim, ai, pos, current_sector, rot, mobj_type) in query.iter() {
		for (player_entity, player_pos, player_sector, player_flags) in players_query.iter() {
			if !player_flags.flags.contains(MobjFlags::SHOOTABLE) {
				continue;
			}

			if !in_fov(pos, rot, player_pos) {
				continue;
			}

			if p_check_sight(
				pos,
				current_sector,
				db.mobjinfo[&mobj_type.type_].height,
				player_pos,
				player_sector,
				PLAYERHEIGHT,
				level,
				traversal,
			) {
				wake_up_monster(
					entity,
					pos,
					mobj_type,
					player_entity,
					anim,
					ai,
					random,
					command_buffer,
					audio_buffer,
					action_buffer,
				);
			}
		}
	}
}

#[allow(clippy::too_many_arguments)]
pub fn chase(
	world: &World,
	ent: Entity,
	random: &mut Random,
	level: &Level,
	game_skill: SkillLevel,
	fast_monsters: bool,
	audio_buffer: &mut Vec<SfxEvent>,
	blocklists: &[Vec<Entity>],
	world_events: &mut Vec<WorldEvent>,
	mobj_flag_buffer: &mut Vec<MobjFlagCommand>,
	action_buffer: &mut Vec<(Entity, ActionFunc)>,
	traversal: &mut Traversal,
) {
	let db = DB.get().unwrap();

	let mut query = world.query_one::<(
		&mut MonsterRotation,
		&mut MobjAi,
		&mut InstantMoveIntent,
		&mut SpriteAnimation,
		&MobjType,
		&Position,
		&CurrentSector,
		&Target,
	)>(ent);
	let Ok((rot, ai, imi, anim, mobj_type, pos, cur_sector, target)) = query.get() else {
		return;
	};

	let mobj_info = &db.mobjinfo[&mobj_type.type_];

	if ai.reaction_time > 0 {
		ai.reaction_time -= 1;
		return;
	}

	let mut target_query =
		world.query_one::<(&Health, &Position, &CurrentSector, &MobjType)>(target.0);
	let Ok((target_hp, target_pos, target_cur_sector, target_type)) = target_query.get() else {
		if let Some(spawn_state) = mobj_info.spawn_state {
			set_mobj_state(ent, ai, anim, spawn_state, action_buffer, 0);
		}
		return;
	};

	if target_hp.0 <= 0 {
		if let Some(spawn_state) = mobj_info.spawn_state {
			set_mobj_state(ent, ai, anim, spawn_state, action_buffer, 0);
		}
		return;
	}

	if ai.threshold > 0 {
		if target_hp.0 <= 0 {
			ai.threshold = 0;
		} else {
			ai.threshold -= 1;
		}
	}

	if mobj_type.flags.contains(MobjFlags::JUST_ATTACKED) {
		mobj_flag_buffer.push(MobjFlagCommand::Remove {
			ent,
			flag: MobjFlags::JUST_ATTACKED,
		});
		if game_skill != SkillLevel::Nightmare && !fast_monsters {
			p_new_chase_dir(
				ent,
				pos,
				rot,
				mobj_type,
				mobj_info,
				imi,
				target_pos,
				level,
				world,
				random,
				blocklists,
				world_events,
				mobj_flag_buffer,
			);
		}
		return;
	}

	let target_info = &db.mobjinfo[&target_type.type_];
	if let Some(melee_state) = mobj_info.melee_state
		&& p_check_melee_range(
			pos,
			cur_sector,
			mobj_info.height,
			target_pos,
			target_cur_sector,
			target_info.height,
			target_info.radius,
			level,
			traversal,
		) {
		if let Some(attack_sound) = &mobj_info.attack_sound {
			audio_buffer.push(SfxEvent {
				sfx_id: to_u64(attack_sound),
				pos: Some((pos.x, pos.y, pos.z)),
			});
		}

		set_mobj_state(ent, ai, anim, melee_state, action_buffer, 0);
		return;
	}

	let mut check_missile = true;

	if let Some(missile_state) = mobj_info.missile_state {
		if game_skill != SkillLevel::Nightmare && !fast_monsters && rot.move_count != 0 {
			check_missile = false;
		}

		if check_missile
			&& p_check_missile_range(
				ent,
				pos,
				cur_sector,
				mobj_type,
				target_pos,
				target_cur_sector,
				target_info.height,
				level,
				traversal,
				random,
				mobj_info.melee_state.is_none(),
				mobj_flag_buffer,
			) {
			set_mobj_state(ent, ai, anim, missile_state, action_buffer, 0);
			return;
		}
	}

	rot.move_count -= 1;
	if rot.move_count < 0
		|| !p_move(
			ent,
			pos,
			rot,
			mobj_type,
			mobj_info,
			imi,
			level,
			world,
			random,
			blocklists,
			world_events,
			mobj_flag_buffer,
		) {
		p_new_chase_dir(
			ent,
			pos,
			rot,
			mobj_type,
			mobj_info,
			imi,
			target_pos,
			level,
			world,
			random,
			blocklists,
			world_events,
			mobj_flag_buffer,
		);

		rot.move_count = (random.p() & 0b111) as i32;
	}

	let Some(active_sound) = &mobj_info.active_sound else {
		return;
	};
	if random.p() < 3 {
		audio_buffer.push(SfxEvent {
			sfx_id: to_u64(active_sound),
			pos: Some((pos.x, pos.y, pos.z)),
		});
	}
}

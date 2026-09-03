use crate::{
	CurrentSector, DB, GameConfig, Health, InstantMoveIntent, MobjAi, MobjFlagCommand, MobjFlags,
	MobjType, MonsterRotation, MoveContext, MoveContextInner, Position, Random, SfxEvent,
	SkillLevel, SpriteAnimation, StateNum, Target, Traversal, WorldEvent, look,
	p_check_melee_range, p_check_missile_range, p_move, p_new_chase_dir,
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
pub fn ai_system(world: &World, actions: &mut Vec<(Entity, ActionFunc)>) {
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
				set_mobj_state(ent, ai, anim, next_state_num, actions, 0);
			}
		}
	}
}

pub fn set_mobj_state(
	ent: Entity,
	ai: &mut MobjAi,
	anim: &mut SpriteAnimation,
	state_num: StateNum,
	actions: &mut Vec<(Entity, ActionFunc)>,
	tics_to_add: i32,
) {
	let db = DB.get().unwrap();

	ai.current_state = state_num;

	let state = db.states[&state_num];
	ai.tics_left = state.tics + tics_to_add;
	anim.cached_rotations = state.cached_rotations;

	if let Some(action) = state.action {
		actions.push((ent, action));
	}
}

pub struct ActionContext<'a> {
	pub world: &'a World,
	pub actions: &'a mut Vec<(Entity, ActionFunc)>,
	pub random: &'a mut Random,
	pub level: &'a mut Level,
	pub cfg: GameConfig,
	pub audio: &'a mut Vec<SfxEvent>,
	pub blocklists: &'a [Vec<Entity>],
	pub world_events: &'a mut Vec<WorldEvent>,
	pub mobj_flags: &'a mut Vec<MobjFlagCommand>,
	pub traversal: &'a mut Traversal,
	pub cmd: &'a mut CommandBuffer,
	pub sound_targets: &'a mut [Option<Entity>],
}

pub fn action_system(mut ctx: ActionContext) {
	std::mem::take(ctx.actions)
		.into_iter()
		.for_each(|(ent, action)| choose_action(&mut ctx, ent, action));
}

fn choose_action(ctx: &mut ActionContext, ent: Entity, action: ActionFunc) {
	match action {
		ActionFunc::Chase => chase(ctx, ent),
		ActionFunc::Look => look(ctx, ent),
		_ => {}
	}
}

pub fn chase(ctx: &mut ActionContext, ent: Entity) {
	let db = DB.get().unwrap();

	let mut query = ctx.world.query_one::<(
		&mut MonsterRotation,
		&mut MobjAi,
		&mut InstantMoveIntent,
		&mut SpriteAnimation,
		&MobjType,
		&Position,
		&CurrentSector,
		&Target,
	)>(ent);
	let Ok((rot, ai, imi, anim, mobj, pos, cur_sector, target)) = query.get() else {
		return;
	};

	let mobj_info = &db.mobjinfo[&mobj.type_];

	if ai.reaction_time > 0 {
		ai.reaction_time -= 1;
		return;
	}

	let mut target_query = ctx
		.world
		.query_one::<(&Health, &Position, &CurrentSector, &MobjType)>(target.0);
	let Ok((target_hp, target_pos, target_cur_sector, target_type)) = target_query.get() else {
		if let Some(spawn_state) = mobj_info.spawn_state {
			set_mobj_state(ent, ai, anim, spawn_state, ctx.actions, 0);
		}
		return;
	};

	if target_hp.0 <= 0 {
		if let Some(spawn_state) = mobj_info.spawn_state {
			set_mobj_state(ent, ai, anim, spawn_state, ctx.actions, 0);
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

	let mut move_ctx = MoveContext {
		ent,
		pos,
		goal_pos: (0.0, pos.y, 0.0),
		mobj,
		mobj_info,
		imi,
		level: ctx.level,
		world: ctx.world,
		random: ctx.random,
		blocklists: ctx.blocklists,
		world_events: ctx.world_events,
		inner: MoveContextInner::default(),
	};

	if mobj.flags.contains(MobjFlags::JUST_ATTACKED) {
		ctx.mobj_flags.push(MobjFlagCommand::Remove {
			ent,
			flag: MobjFlags::JUST_ATTACKED,
		});
		if ctx.cfg.skill != SkillLevel::Nightmare && !ctx.cfg.fast_monsters {
			p_new_chase_dir(&mut move_ctx, rot, target_pos, ctx.mobj_flags);
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
			ctx.level,
			ctx.traversal,
		) {
		if let Some(attack_sound) = &mobj_info.attack_sound {
			ctx.audio.push(SfxEvent {
				sfx_id: to_u64(attack_sound),
				pos: Some((pos.x, pos.y, pos.z)),
			});
		}

		set_mobj_state(ent, ai, anim, melee_state, ctx.actions, 0);
		return;
	}

	let mut check_missile = true;

	if let Some(missile_state) = mobj_info.missile_state {
		if ctx.cfg.skill != SkillLevel::Nightmare && !ctx.cfg.fast_monsters && rot.move_count != 0 {
			check_missile = false;
		}

		if check_missile
			&& p_check_missile_range(
				ent,
				pos,
				cur_sector,
				mobj,
				target_pos,
				target_cur_sector,
				target_info.height,
				ctx.level,
				ctx.traversal,
				move_ctx.random,
				mobj_info.melee_state.is_none(),
				ctx.mobj_flags,
			) {
			set_mobj_state(ent, ai, anim, missile_state, ctx.actions, 0);
			return;
		}
	}

	rot.move_count -= 1;
	if rot.move_count < 0 || !p_move(&mut move_ctx, rot, ctx.mobj_flags) {
		p_new_chase_dir(&mut move_ctx, rot, target_pos, ctx.mobj_flags);

		rot.move_count = (ctx.random.p() & 0b111) as i32;
	}

	if let Some(active_sound) = &mobj_info.active_sound
		&& ctx.random.p() < 3
	{
		ctx.audio.push(SfxEvent {
			sfx_id: to_u64(active_sound),
			pos: Some((pos.x, pos.y, pos.z)),
		});
	};
}

use crate::{
	Action, Collider, CurrentSector, DB, Database, GameConfig, Health, InstantMoveIntent,
	LookComponents, MobjAi, MobjFlagCommand, MobjFlags, MobjType, MonsterRotation, MoveContext,
	MoveContextInner, Position, Random, SfxEvent, SightContext, SkillLevel, SpriteAnimation,
	StateNum, Target, Traversal, WorldEvent, look, p_check_melee_range, p_check_missile_range,
	p_move, p_new_chase_dir,
};
use hecs::{CommandBuffer, Entity, QueryIter, World};
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
pub fn ai_system(world: &World) {
	let db = DB.get().unwrap();

	let mut query = world.query::<(&mut MobjAi, &mut SpriteAnimation, &mut Action)>();
	for (ai, anim, act) in query.iter() {
		if ai.tics_left <= 0 {
			continue;
		}

		ai.tics_left -= 1;
		if ai.tics_left == 0 {
			let current_state = db.states[ai.current_state as usize];

			if let Some(next_state_num) = current_state.next_state {
				set_mobj_state(act, ai, anim, next_state_num, db, 0);
			}
		}
	}
}

pub(crate) fn set_mobj_state(
	action: &mut Action,
	ai: &mut MobjAi,
	anim: &mut SpriteAnimation,
	state_num: StateNum,
	db: &Database,
	tics_to_add: i32,
) {
	ai.current_state = state_num;

	let state = db.states[state_num as usize];
	ai.tics_left = state.tics + tics_to_add;
	anim.cached_rotations = state.cached_rotations;
	action.0 = state.action;
}

pub(crate) struct ActionContext<'a> {
	pub world: &'a World,
	pub random: &'a mut Random,
	pub level: &'a mut Level,
	pub cfg: GameConfig,
	pub audio: &'a mut Vec<SfxEvent>,
	pub blocklists: &'a [Vec<(Entity, Collider)>],
	pub world_events: &'a mut Vec<WorldEvent>,
	pub mobj_flags: &'a mut Vec<MobjFlagCommand>,
	pub traversal: &'a mut Traversal,
	pub cmd: &'a mut CommandBuffer,
	pub sound_targets: &'a mut [Option<Entity>],
	pub db: &'a Database,
}

#[allow(clippy::too_many_arguments)]
pub fn action_system(
	world: &World,
	random: &mut Random,
	level: &mut Level,
	cfg: GameConfig,
	audio: &mut Vec<SfxEvent>,
	blocklists: &[Vec<(Entity, Collider)>],
	world_events: &mut Vec<WorldEvent>,
	mobj_flags: &mut Vec<MobjFlagCommand>,
	traversal: &mut Traversal,
	cmd: &mut CommandBuffer,
	sound_targets: &mut [Option<Entity>],
) {
	let db = DB.get().unwrap();

	let mut ctx = ActionContext {
		world,
		random,
		level,
		cfg,
		audio,
		blocklists,
		world_events,
		mobj_flags,
		traversal,
		cmd,
		sound_targets,
		db,
	};

	let mut chase_query = ctx.world.query::<ChaseComponents>();
	chase(&mut ctx, chase_query.iter());
	drop(chase_query);

	let mut look_query = ctx.world.query::<LookComponents>();
	look(&mut ctx, look_query.iter());
}

type ChaseComponents<'a> = (
	Entity,
	&'a mut MonsterRotation,
	&'a mut MobjAi,
	&'a mut InstantMoveIntent,
	&'a mut SpriteAnimation,
	&'a MobjType,
	&'a Position,
	&'a CurrentSector,
	&'a Target,
	&'a mut Action,
);

pub(crate) fn chase(ctx: &mut ActionContext, query: QueryIter<'_, ChaseComponents>) {
	for (ent, rot, ai, imi, anim, mobj, pos, cur_sector, target, act) in query
		.filter(|(.., act)| act.0 == Some(ActionFunc::Chase))
		.map(|(_e, _r, _ai, _i, _an, m, p, s, t, _ac)| (_e, _r, _ai, _i, _an, *m, *p, *s, *t, _ac))
	{
		let mobj_info = &ctx.db.mobjinfo[mobj.type_ as usize];

		if ai.reaction_time > 0 {
			ai.reaction_time -= 1;
			return;
		}

		let Ok((target_hp, target_pos, target_cur_sector, target)) = ctx
			.world
			.query_one::<(&Health, &Position, &CurrentSector, &MobjType)>(target.0)
			.get()
			.map(|(h, p, s, t)| (*h, *p, *s, *t))
		else {
			if let Some(spawn_state) = mobj_info.spawn_state {
				set_mobj_state(act, ai, anim, spawn_state, ctx.db, 0);
			}
			return;
		};

		if target_hp.0 <= 0 {
			if let Some(spawn_state) = mobj_info.spawn_state {
				set_mobj_state(act, ai, anim, spawn_state, ctx.db, 0);
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
			random: ctx.random,
			blocklists: ctx.blocklists,
			world_events: ctx.world_events,
			db: ctx.db,
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

		let target_info = &ctx.db.mobjinfo[target.type_ as usize];
		let sight_ctx = SightContext {
			pos,
			cur_sector,
			height: mobj_info.height,
			target_pos,
			target_sector: target_cur_sector,
			target_height: target_info.height,
			level: ctx.level,
		};

		if let Some(melee_state) = mobj_info.melee_state
			&& p_check_melee_range(&sight_ctx, ctx.traversal, target_info.radius)
		{
			if let Some(attack_sound) = &mobj_info.attack_sound {
				ctx.audio.push(SfxEvent {
					sfx_id: to_u64(attack_sound),
					pos: Some((pos.x, pos.y, pos.z)),
				});
			}

			set_mobj_state(act, ai, anim, melee_state, ctx.db, 0);
			return;
		}

		let mut check_missile = true;

		if let Some(missile_state) = mobj_info.missile_state {
			if ctx.cfg.skill != SkillLevel::Nightmare
				&& !ctx.cfg.fast_monsters
				&& rot.move_count != 0
			{
				check_missile = false;
			}

			if check_missile
				&& p_check_missile_range(
					&sight_ctx,
					ent,
					mobj,
					ctx.traversal,
					move_ctx.random,
					ctx.mobj_flags,
					mobj_info.melee_state.is_none(),
				) {
				set_mobj_state(act, ai, anim, missile_state, ctx.db, 0);
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
}

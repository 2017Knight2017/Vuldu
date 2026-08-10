use hecs::{CommandBuffer, Entity, QueryBorrow, World};
use serde::Deserialize;
use wad_parser::{Level, to_u64};
use crate::{CurrentSector, DB, Health, Idle, InstantMoveIntent, MobjAi, MobjFlagCommand, MobjFlags, MobjType, MonsterRotation, PlayerMarker, Position, Random, SfxEvent, SkillLevel, SpriteAnimation, Target, Traversal, WorldEvent, in_fov, p_check_melee_range, p_check_missile_range, p_check_sight, p_move, p_new_chase_dir, wake_up_monster};

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
	let mut query = world.query::<&mut MobjAi>();
	for ai in query.iter() {
		if ai.tics_left <= 0 {
			continue;
		}

		ai.tics_left -= 1;
		if ai.tics_left == 0 {
			let db = DB.get().unwrap();

			let current_state = db.states[&ai.current_state];

			if let Some(next_state_num) = current_state.next_state {
				let next_state = db.states[&next_state_num];

				ai.current_state = next_state_num;
				ai.tics_left = next_state.tics;
				ai.action = next_state.action;
			}
		}
	}
}

/// Must be called after propagate_sound_system
pub fn check_sound_system( 
	world: &World,
	level: &mut Level,
	random: &mut Random,
	command_buffer: &mut CommandBuffer, 
	sound_targets: &mut [Option<Entity>],
	traversal: &mut Traversal,
	audio_buffer: &mut Vec<SfxEvent>, 
) {
	let db = DB.get().unwrap();

	let mut query = world
		.query::<(Entity, &mut SpriteAnimation, &mut MobjAi, &Position, &CurrentSector, &MobjType)>()
		.with::<&Idle>();
	for (entity, sprite_anim, ai, pos, current_sector, mobj_type) in query.iter() {
        if let Some(sound_target_entity) = sound_targets[current_sector.0.0] {            
            if mobj_type.flags.contains(MobjFlags::AMBUSH) {
				let mut target_query = world.query_one::<(&Position, &CurrentSector)>(sound_target_entity);

                match target_query.get() {
                    Ok((target_pos, target_sector)) => {
						if !p_check_sight( 
							pos, 
							current_sector,
							db.mobjinfo[&mobj_type.type_].height,
							target_pos,
							target_sector,
							level,
							traversal
						) {
                	        continue;
                	    }
					}
					Err(_) => continue
                } 
            }

            wake_up_monster(
                entity, 
                pos, 
                mobj_type, 
				sound_target_entity,
				sprite_anim,
				ai, 
                random, 
                command_buffer, 
                audio_buffer,
                db
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
) {
	
	let db = DB.get().unwrap();

	let mut query = world
        .query::<(Entity, &mut SpriteAnimation, &mut MobjAi, &Position, &CurrentSector, &MonsterRotation, &MobjType)>()
        .with::<&Idle>();
	let mut players_query = world
        .query::<(Entity, &Position, &CurrentSector, &MobjType)>()
        .with::<&PlayerMarker>();

	for (entity, sprite_anim, ai, pos, current_sector, rot, mobj_type) in query.iter() {
        for (player_entity, player_pos, player_sector, player_flags) in players_query.iter() {
            
            if !player_flags.flags.contains(MobjFlags::SHOOTABLE) {
                continue;
            }

            if !in_fov(&pos, &rot, &player_pos) {
                continue;
            }

            if p_check_sight( 
				pos, 
				current_sector,
				db.mobjinfo[&mobj_type.type_].height,
				player_pos,
				player_sector,
				level,
				traversal
			) {
            	wake_up_monster(
            	    entity, 
            	    pos, 
            	    mobj_type, 
					player_entity,
					sprite_anim,
					ai, 
            	    random, 
            	    command_buffer, 
            	    audio_buffer,
            	    db
            	);
			}
        }
    }
}

pub fn chase_system(
	mut query: QueryBorrow<'_, (
		Entity,
        &mut MonsterRotation,
		&mut MobjAi,
		&mut InstantMoveIntent,
		&MobjType,
		&Position,
        &Target,
    )>,
	world: &World,
    random: &mut Random,
    map: &Level,
    game_skill: SkillLevel,
    fast_monsters: bool,
    audio_buffer: &mut Vec<SfxEvent>,
	blocklists: &[Vec<Entity>],
	world_events: &mut Vec<WorldEvent>,
	mobj_flag_buffer: &mut Vec<MobjFlagCommand>
) {
    let db = DB.get().unwrap();

    for (ent, rot, ai, imi, mobj_type, pos, target) in query.iter() {
        let mobj_info = &db.mobjinfo[&mobj_type.type_];

        if ai.reaction_time > 0 {
            ai.reaction_time -= 1;
        }

		let mut target_query = world.query_one::<(&Health, &Position)>(target.0);
		let (target_hp, target_pos) = match target_query.get() {
			Ok(data) => data,
			Err(_) => {
				if let Some(spawn_state) = mobj_info.spawn_state {
					ai.current_state = spawn_state;
                	ai.tics_left = db.states[&spawn_state].tics;
            	}
            	continue;
			}
		};

		if target_hp.0 <= 0 {
            if let Some(spawn_state) = mobj_info.spawn_state {
				ai.current_state = spawn_state;
                ai.tics_left = db.states[&spawn_state].tics;
            }
            continue;
        }

        if ai.threshold > 0 {
            if target_hp.0 <= 0 {
                ai.threshold = 0;
            } else {
                ai.threshold -= 1;
            }
        }

        if mobj_type.flags.contains(MobjFlags::JUST_ATTACKED) {
            mobj_flag_buffer.push(MobjFlagCommand::Remove { ent, flag: MobjFlags::JUST_ATTACKED });
            if game_skill != SkillLevel::Nightmare && !fast_monsters {
                p_new_chase_dir(
    			    ent, pos, rot, mobj_type, mobj_info, imi, target_pos,
    			    map, world, random, blocklists, world_events, mobj_flag_buffer
    			);
            }
            continue;
        }

        if let Some(melee_state) = mobj_info.melee_state {
            if p_check_melee_range(pos, target_pos, mobj_info.radius) {
                if let Some(attack_sound) = &mobj_info.attack_sound {
                    audio_buffer.push(SfxEvent {
                        sfx_id: to_u64(attack_sound),
                        pos: Some((pos.x, pos.y, pos.z)),
                    });
                }

				ai.current_state = melee_state;
                ai.tics_left = db.states[&melee_state].tics;

                continue;
            }
        }

        let mut check_missile = true;

        if let Some(missile_state) = mobj_info.missile_state {
            if game_skill != SkillLevel::Nightmare && !fast_monsters && rot.move_count != 0 {
                check_missile = false;
            }

            if check_missile && p_check_missile_range(
				ent,
				pos, 
				mobj_type, 
				target_pos, 
				random, 
				mobj_info.melee_state.is_none(),
				mobj_flag_buffer
			) {
				ai.current_state = missile_state;
                ai.tics_left = db.states[&missile_state].tics;
				mobj_flag_buffer.push(MobjFlagCommand::Add { ent, flag: MobjFlags::JUST_ATTACKED }); 
                continue;
            }
        }

        rot.move_count -= 1;
        if rot.move_count < 0 || !p_move(
			ent, pos, rot, mobj_type, mobj_info, imi, map, world,
			random, blocklists, world_events, mobj_flag_buffer
		) {
            p_new_chase_dir(
    		    ent, pos, rot, mobj_type, mobj_info, imi, target_pos,
    		    map, world, random, blocklists, world_events, mobj_flag_buffer
    		);
		
    		rot.move_count = (random.p() & 15) as i32;
        }

        if let Some(active_sound) = &mobj_info.active_sound {
            if random.p() < 3 {
                audio_buffer.push(SfxEvent {
            	    sfx_id: to_u64(active_sound),
            	    pos: Some((pos.x, pos.y, pos.z)),
            	});
            }
        }

    }
}

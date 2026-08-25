use std::collections::VecDeque;

use engine::{ActionFunc, GameConfig, GameState, MobjFlagCommand, PlayerInput, Random, Traversal, UpdatableUiType, WorldEvent, action_system, ai_system, animation_system, apply_mobj_flags_system, apply_monster_movement_system, apply_player_movement_system, check_sound_system, execute_events_system, friction_system, handle_position_input, handle_rotation_input, handle_weapons_input, propagate_sound_system, try_move_system};
use hecs::{CommandBuffer, Entity, World};
use wad_parser::Level;
use winit::keyboard::PhysicalKey;

use crate::{cheats::cheat_system, sound_player::AudioContext};

pub struct GameContext {
    pub world: World,
    pub level: Level,
    pub state: GameState,
    pub config: GameConfig,
    pub blocklists: Vec<Vec<Entity>>,
    sound_targets: Vec<Option<Entity>>,
    world_events: Vec<WorldEvent>,
    mobj_flag_buffer: Vec<MobjFlagCommand>,
    action_buffer: Vec<(Entity, ActionFunc)>,
    command_buffer: CommandBuffer,
    traversal: Traversal,
    pub player_entity: Entity,
    pub global_timer: u32
}

impl GameContext {
    pub fn new(level: Level, config: GameConfig) -> Self {
        Self { 
            world: World::new(), 
            sound_targets: vec![None; level.state.sectors.len()],
            blocklists: vec![Vec::new(); level.geom.blockmap.row_num * level.geom.blockmap.col_num], 
            traversal: Traversal::for_level(&level),
            level, 
            world_events: Vec::new(), 
            mobj_flag_buffer: Vec::new(), 
            command_buffer: CommandBuffer::new(),
            action_buffer: Vec::new(),
            player_entity: Entity::DANGLING,
            state: GameState::Level,
            global_timer: 0,
            config,
        }
    }

    pub fn tick(
        &mut self, 
        audio: &mut AudioContext, 
        current_input: &mut PlayerInput, 
        random: &mut Random, 
        ui_to_update: &mut Vec<UpdatableUiType>,
        last_buttons: &mut VecDeque<Option<PhysicalKey>>
    ) {
        handle_rotation_input(&self.world, self.player_entity, current_input);
        handle_position_input(&self.world, self.player_entity, current_input);
        handle_weapons_input(&self.world, self.player_entity, ui_to_update, 
            &mut self.command_buffer, &mut audio.buffer, current_input);

        self.flush_command_buffer();

        propagate_sound_system(&self.world, &self.level, &mut self.sound_targets,
            &mut self.traversal, &mut self.command_buffer);

        self.flush_command_buffer();

        check_sound_system(&self.world, &mut self.level, random, &mut self.command_buffer,
            &mut self.sound_targets, &mut self.traversal, &mut audio.buffer, &mut self.action_buffer);

        //check_sight_system(&self.world, &self.level, &mut self.traversal, random, 
        //    &mut self.command_buffer, &mut audio.buffer, &mut self.action_buffer);

        self.flush_command_buffer();

        ai_system(&self.world, &mut self.action_buffer);
        action_system(&self.world, &mut self.action_buffer, random, &self.level, self.config.skill, 
            self.config.fast_monsters, &mut audio.buffer, &self.blocklists, &mut self.world_events, 
            &mut self.mobj_flag_buffer);

        friction_system(&self.world);

        let pending_moves = try_move_system(&self.world, &self.level,
            random, &self.blocklists, &mut self.world_events);

        apply_player_movement_system(&self.world, &self.level);
        apply_monster_movement_system(&self.world, pending_moves, &self.level, &mut self.blocklists);

        cheat_system(last_buttons, &mut self.world_events);

        execute_events_system(&mut self.world_events, &self.world, self.player_entity, ui_to_update);
        apply_mobj_flags_system(&mut self.mobj_flag_buffer, &self.world);

        audio.system(&self.world);
        animation_system(&self.world);

        current_input.mouse_delta_x = 0.0;
    }

    fn flush_command_buffer(&mut self) {
        self.command_buffer.run_on(&mut self.world);
        self.command_buffer.clear();
    }
}

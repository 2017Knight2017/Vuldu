use std::collections::VecDeque;

use engine::*;
use hecs::{CommandBuffer, Entity, World};
use wad_parser::Level;
use winit::keyboard::PhysicalKey;

use crate::{cheats::cheat_system, sound_player::AudioContext};

pub struct GameContext {
	pub world: World,
	pub level: Level,
	pub state: GameState,
	pub config: GameConfig,
	pub blocklists: Vec<Vec<(Entity, Collider)>>,
	pub graphics_buffer: Vec<GraphicsCommand>,
	intercepts: Vec<Intercept>,
	buttons: Vec<Button>,
	sound_targets: Vec<Option<Entity>>,
	world_events: Vec<WorldEvent>,
	mobj_flags: Vec<MobjFlagCommand>,
	actions: Vec<(Entity, ActionFunc)>,
	cmd: CommandBuffer,
	traversal: Traversal,
	pub player_entity: Entity,
	pub global_timer: u32,
}

impl GameContext {
	pub fn new(level: Level, config: GameConfig) -> Self {
		Self {
			world: World::new(),
			sound_targets: vec![None; level.state.sectors.len()],
			blocklists: vec![Vec::new(); level.geom.blockmap.row_num * level.geom.blockmap.col_num],
			traversal: Traversal::for_level(&level),
			level,
			graphics_buffer: Vec::new(),
			world_events: Vec::new(),
			mobj_flags: Vec::new(),
			intercepts: Vec::new(),
			buttons: Vec::new(),
			cmd: CommandBuffer::new(),
			actions: Vec::new(),
			player_entity: Entity::DANGLING,
			state: GameState::Level,
			global_timer: 0,
			config,
		}
	}

	pub fn tick(
		&mut self,
		audio: &mut AudioContext,
		input: PlayerInput,
		random: &mut Random,
		ui_to_update: &mut Vec<UpdatableUiType>,
		last_buttons: &mut VecDeque<Option<PhysicalKey>>,
	) {
		handle_rotation_input(&self.world, self.player_entity, input.mouse_delta_x);
		handle_position_input(&self.world, self.player_entity, input);
		handle_weapons_input(
			&self.world,
			self.player_entity,
			ui_to_update,
			&mut self.cmd,
			&mut audio.buffer,
			input,
		);
		handle_use_input(
			&self.world,
			self.player_entity,
			input.use_,
			&mut self.level,
			&mut self.traversal,
			&mut self.intercepts,
			&mut self.buttons,
			&mut audio.buffer,
		);

		self.flush_command_buffer();

		propagate_sound_system(
			&self.world,
			&self.level,
			&mut self.sound_targets,
			&mut self.traversal,
			&mut self.cmd,
		);

		self.flush_command_buffer();

		ai_system(&self.world, &mut self.actions);

		action_system(
			&self.world,
			&mut self.actions,
			random,
			&mut self.level,
			self.config,
			&mut audio.buffer,
			&self.blocklists,
			&mut self.world_events,
			&mut self.mobj_flags,
			&mut self.traversal,
			&mut self.cmd,
			&mut self.sound_targets,
		);

		friction_system(&self.world);

		let pending_moves = try_move_system(
			&self.world,
			&self.level,
			random,
			&self.blocklists,
			&mut self.world_events,
		);

		apply_player_movement_system(&self.world, &self.level);
		apply_monster_movement_system(
			&self.world,
			pending_moves,
			&self.level,
			&mut self.blocklists,
		);

		cheat_system(last_buttons, &mut self.world_events);

		execute_events_system(
			&mut self.world_events,
			&self.world,
			&self.level,
			self.player_entity,
			ui_to_update,
			&mut self.cmd,
			&mut audio.buffer,
			&mut self.blocklists,
			&mut self.graphics_buffer,
			self.config,
			self.global_timer,
		);
		apply_mobj_flags_system(&mut self.mobj_flags, &self.world);

		button_system(&mut self.level, &mut self.buttons, &mut audio.buffer);
		animation_system(&self.world);
		audio.system(&self.world, self.player_entity);
		update_blocklists_system(&self.world, &self.level, &mut self.blocklists);
	}

	fn flush_command_buffer(&mut self) {
		self.cmd.run_on(&mut self.world);
		self.cmd.clear();
	}
}

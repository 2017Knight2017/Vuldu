mod graphics;
mod parse_commandline;
mod sound_player;

//use parse_commandline::Args;
use sound_player::*;
use renderer::*;
use wad_parser::map::Level;
use wad_parser::*;
use engine::*;
use hecs::{CommandBuffer, Entity, World};
use winit::{
    application::ApplicationHandler, 
    dpi::LogicalSize, 
    error::OsError, 
    event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent}, 
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::{CursorGrabMode, Window, WindowId}
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
//use clap::Parser;
use std::time::Instant;
use crate::graphics::GraphicsContext;

const TICKRATE: u32 = 35;
const TICK_TIME: f32 = 1.0 / TICKRATE as f32;

struct GameContext {
    world: World,
    level: Level,
    blocklists: Vec<Vec<Entity>>,
    sound_targets: Vec<Option<Entity>>,
    world_events: Vec<WorldEvent>,
    mobj_flag_buffer: Vec<MobjFlagCommand>,
    command_buffer: CommandBuffer,
    traversal: Traversal
}

struct App {
    window: Option<Window>,
    wad_manager: WadManager,
    graphics: GraphicsContext,
    game: GameContext,
    audio: AudioContext,
    current_input: PlayerInput,
    random: Random,
    is_shutting_down: bool,
    last_frame_time: Instant,
    time_accumulator: f32,
}

impl GameContext {
    fn new(level: Level) -> Self {
        Self { 
            world: World::new(), 
            sound_targets: vec![None; level.state.sectors.len()],
            blocklists: vec![Vec::new(); level.geom.blockmap.row_num * level.geom.blockmap.col_num], 
            traversal: Traversal::for_level(&level),
            level, 
            world_events: Vec::new(), 
            mobj_flag_buffer: Vec::new(), 
            command_buffer: CommandBuffer::new() 
        }
    }

    fn tick(&mut self, audio: &mut AudioContext, current_input: &mut PlayerInput, random: &mut Random) {
        handle_rotation_input(&self.world, current_input);
        handle_position_input(&self.world, current_input, &mut self.command_buffer, &mut audio.buffer);

        self.flush_command_buffer();

        propagate_sound_system(
            &self.world, 
            &self.level,
            &mut self.sound_targets,
            &mut self.traversal,
            &mut self.command_buffer, 
        );

        self.flush_command_buffer();

        check_sound_system(
            &self.world, 
            &mut self.level, 
            random, 
            &mut self.command_buffer, 
            &mut self.sound_targets,
            &mut self.traversal,
            &mut audio.buffer,
        );

        //check_sight_system(
        //    check_sight_query, 
        //    players_query, 
        //    &self.level, 
        //    &mut self.traversal,
        //    &mut self.random, 
        //    &mut self.command_buffer, 
        //    &mut self.audio.buffer
        //);

        self.flush_command_buffer();

        let chase_query = self.world.query::<(
            Entity, &mut MonsterRotation, &mut MobjAi,  
            &mut InstantMoveIntent, &MobjType, &Position, &Target
        )>();
        chase_system(
            chase_query, 
            &self.world, 
            random, 
            &self.level, 
            SkillLevel::Hard, 
            true, 
            &mut audio.buffer, 
            &self.blocklists, 
            &mut self.world_events,
            &mut self.mobj_flag_buffer,
        );

        friction_system(&self.world);

        let pending_moves = try_move_system(
            &self.world,
            &self.level,
            random,
            &mut self.blocklists,
            &mut self.world_events
        );

        apply_player_movement_system(&self.world, &self.level);
        apply_monster_movement_system(&self.world, pending_moves, &self.level, &mut self.blocklists);

        execute_events_system(&mut self.world_events);
        apply_mobj_flags_system(&mut self.mobj_flag_buffer, &self.world);

        audio.system(&self.world);
        ai_system(&self.world);
        animation_system(&self.world);

        current_input.mouse_delta_x = 0.0;
    }

    fn flush_command_buffer(&mut self) {
        self.command_buffer.run_on(&mut self.world);
        self.command_buffer.clear();
    }
}

impl App {
    fn create_window(&self, event_loop: &ActiveEventLoop) -> Result<Window, OsError> {
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(1280, 720))
            .with_title("Vuldu")
            .with_visible(true);

        let window = event_loop.create_window(window_attributes)?;
        window.set_cursor_visible(false);

        if let Err(err) = window.set_cursor_grab(CursorGrabMode::Locked) {
            let _ = window.set_cursor_grab(CursorGrabMode::Confined);
            eprintln!("Failed to lock cursor: {:?}", err);
        }

        Ok(window)
    }

    fn handle_fatal_error(&mut self, event_loop: &ActiveEventLoop, renderer: &mut SafeRenderer, msg: &str) {
        eprintln!("[FATAL] {}", msg);
        self.is_shutting_down = true;
        renderer.shutdown();
        event_loop.exit();
    }

    fn update_game_logic(&mut self) {
        while self.time_accumulator >= TICK_TIME {
            self.game.tick(&mut self.audio, &mut self.current_input, &mut self.random);
            self.time_accumulator -= TICK_TIME;
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match self.create_window(event_loop) {
            Ok(win) => win,
            Err(err) => {
                eprintln!("Failed to create window: {:?}", err);
                event_loop.exit();
                return;
            }
        };
        self.window = Some(window);

        self.audio.data = self.wad_manager.bake_sfx();
        let mut renderer = SafeRenderer::new();

        let window_ref = self.window.as_ref().unwrap();
        let display_handle = window_ref.display_handle().unwrap().as_raw();
        let window_handle = window_ref.window_handle().unwrap().as_raw();

        if let (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) = (display_handle, window_handle) {
            let handles = WindowHandles {
                display_ptr: d.display.as_ptr() as usize,
                window_ptr: w.surface.as_ptr() as usize,
            };

            let window_raw_ptr = window_ref as *const Window as usize;
            renderer.init(&handles, window_raw_ptr);
            renderer.set_resolution(1280, 720);

            let sky_idx = if self.wad_manager.is_doom1 { 
                (self.game.level.map_num as u32 - 1) / 9 
            } else { 
                get_sky_texture_index(self.game.level.map_num) 
            };
            renderer.set_sky_index(sky_idx);

            if let Err(err) = self.graphics.load_and_upload_textures(&mut renderer, &self.wad_manager, self.game.level.map_num) {
                self.handle_fatal_error(event_loop, &mut renderer, &err);
                return;
            }

            self.game.level.geom.sector_lines = self.graphics.setup_level_geometry(&mut renderer, &self.game.level);

            let _ = engine::populate_database(&self.graphics.data).map_err(|e| eprintln!("{}", e));
            engine::spawn_all_things(&mut self.game.world, &self.game.level, &mut self.random);
            println!("Mobj spawning is done!");

            self.graphics.renderer = Some(renderer);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.current_input.mouse_delta_x += delta.0 as f32;
            },

            _ => {}
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(win_size) => {
                if let Some(renderer) = &mut self.graphics.renderer {
                    renderer.recreate_swapchain();
                    renderer.set_resolution(win_size.width, win_size.height);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat { return; }

                let is_pressed = event.state == ElementState::Pressed;
                
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.current_input.move_forward = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.current_input.move_backward = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.current_input.move_left = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.current_input.move_right = is_pressed,
                    PhysicalKey::Code(KeyCode::Space) => self.current_input.move_up = is_pressed,
                    PhysicalKey::Code(KeyCode::ShiftLeft) => self.current_input.move_down = is_pressed,
                    _ => {}
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = state == ElementState::Pressed;

                match button {
                    MouseButton::Left => self.current_input.shoot = is_pressed,
                    _ => {}
                }
            }

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                self.is_shutting_down = true;
                if let Some(renderer) = &mut self.graphics.renderer {
                    renderer.shutdown();
                }
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if self.is_shutting_down { 
                    return; 
                }
            
                let current_time = Instant::now();
                let delta_time = current_time.duration_since(self.last_frame_time).as_secs_f32().min(0.25);
                self.last_frame_time = current_time;
                self.time_accumulator += delta_time;
            
                self.update_game_logic();
            
                if let Some(window) = &self.window {
                    let alpha = self.time_accumulator / TICK_TIME;
                    self.graphics.render(window, &self.game.world, &self.game.level, alpha);

                    window.request_redraw();
                }
            }

            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn get_sky_texture_index(map: u8) -> u32 {
    if map <= 11 {
        0
    } else if map <= 20 {
        1
    } else {
        2
    }
}

fn main() -> Result<(), String> {
    //let args = Args::parse();
    let mut wad_manager = WadManager::new();

    //wad_manager.add_wad(args.iwad)?;
    //for pwad in args.pwads {
    //    wad_manager.add_wad(pwad)?;
    //}
    //
    //let map = Level::from_wad(&wad_manager, &args.map)?;

    wad_manager.add_wad("assets/DOOM2.WAD")?;
    //wad_manager.add_wad("assets/test.wad")?;
    //wad_manager.add_wad("assets/oku2v31.wad")?;
    //wad_manager.add_wad("assets/nuts.wad")?;
    //wad_manager.add_wad("assets/Sunder 2512.wad")?;
    //wad_manager.add_wad("assets/HR.WAD")?;

    let level = Level::load(&wad_manager, 1)?;

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        wad_manager,
        graphics: GraphicsContext::new(),
        game: GameContext::new(level),
        audio: AudioContext::new()?,
        random: Random::default(),
        is_shutting_down: false,
        current_input: PlayerInput::default(),
        last_frame_time: Instant::now(),
        time_accumulator: 0.0,
    };
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}

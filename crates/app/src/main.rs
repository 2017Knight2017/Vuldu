mod graphics;
mod parse_commandline;
mod sound_player;
mod cheats;

use crate::{cheats::cheat_system, graphics::{GraphicsContext, GraphicsFlags}};
use parse_commandline::Args;
use sound_player::*;
use renderer::*;
use wad_parser::map::Level;
use wad_parser::*;
use engine::*;
use hecs::{CommandBuffer, Entity, World};
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, error::OsError, event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, Window, WindowId}
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use clap::Parser;
use std::{collections::VecDeque, error::Error, time::Instant};

#[cfg(target_os = "macos")]
use objc2::{msg_send, runtime::AnyObject};

const TICKRATE: u32 = 35;
const TICK_TIME: f32 = 1.0 / TICKRATE as f32;

struct GameContext {
    world: World,
    level: Level,
    state: GameState,
    skill: SkillLevel,
    fast_monsters: bool,
    blocklists: Vec<Vec<Entity>>,
    sound_targets: Vec<Option<Entity>>,
    world_events: Vec<WorldEvent>,
    mobj_flag_buffer: Vec<MobjFlagCommand>,
    action_buffer: Vec<(Entity, ActionFunc)>,
    command_buffer: CommandBuffer,
    traversal: Traversal,
    player_entity: Entity,
    global_timer: u32
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
    last_buttons_pressed: VecDeque<Option<PhysicalKey>>
}

impl GameContext {
    fn new(level: Level, skill: SkillLevel, fast_monsters: bool) -> Self {
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
            skill,
            fast_monsters
        }
    }

    fn tick(
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
        action_system(&self.world, &mut self.action_buffer, random, &self.level, self.skill, 
            self.fast_monsters, &mut audio.buffer, &self.blocklists, &mut self.world_events, 
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

    fn update_game_logic(&mut self) -> Result<(), Box<dyn Error>> {
        while self.time_accumulator >= TICK_TIME {
            self.game.tick(
                &mut self.audio, 
                &mut self.current_input, 
                &mut self.random, 
                &mut self.graphics.ui_to_update,
                &mut self.last_buttons_pressed,
            );
            self.time_accumulator -= TICK_TIME;
            self.game.global_timer = self.game.global_timer.wrapping_add(1);
        }

        Ok(())
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

        let handles = match (display_handle, window_handle) {
            (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) => 
                WindowHandles {
                    display_ptr: d.display.as_ptr() as usize,
                    window_ptr: w.surface.as_ptr() as usize,
                    is_x11: false,
                },
            (RawDisplayHandle::Xlib(d), RawWindowHandle::Xlib(w)) => 
                WindowHandles {
                    display_ptr: d.display.unwrap().as_ptr() as usize,
                    window_ptr: w.window as usize,
                    is_x11: true,
                },
            (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(w)) => 
                WindowHandles {
                    display_ptr: w.hinstance.map(|handle| handle.get()).unwrap_or(0) as usize,
                    window_ptr: w.hwnd.get() as usize,
                    is_x11: false,
                },
            #[cfg(target_os = "macos")]
            (RawDisplayHandle::AppKit(_), RawWindowHandle::AppKit(w)) => {
                let view_ptr = w.ns_view.as_ptr() as *mut AnyObject;

                let layer_class = objc2::runtime::AnyClass::get(c"CAMetalLayer")
                    .expect("CAMetalLayer class not found");

                let layer: *mut AnyObject = unsafe { msg_send![layer_class, layer] };
                
                unsafe {
                    let _: () = msg_send![view_ptr, setLayer: layer];
                    let _: () = msg_send![view_ptr, setWantsLayer: true];
                }

                WindowHandles {
                    display_ptr: 0,
                    window_ptr: layer as usize,
                    is_x11: false,
                }
            },
            (_, _) => {
                eprintln!("Unsupported platform. Available platforms are: Linux Wayland, Linux X11, Windows, MacOS.");
                event_loop.exit();
                return;
            }
        };

        let window_raw_ptr = window_ref as *const Window as usize;
        renderer.init(&handles, window_raw_ptr);

        let sky_idx = if self.wad_manager.is_doom1 { 
            (self.game.level.map_num as u32 - 1) / 9 
        } else { 
            get_sky_texture_index(self.game.level.map_num) 
        };
        renderer.set_sky_index(sky_idx);

        if let Err(err) = self.graphics.load_and_upload_textures(&mut renderer, &self.wad_manager, self.game.level.map_num) {
            handle_fatal_error(&mut self.is_shutting_down, event_loop, &mut Some(renderer), &err);
            return;
        }

        self.graphics.setup_level_geometry(&mut renderer, &mut self.game.level);

        let _ = engine::populate_database(&self.graphics.data).map_err(|e| eprintln!("{}", e));
        engine::spawn_all_things(&mut self.game.world, &self.game.level, 
            &mut self.random, &mut self.game.player_entity, &mut self.game.blocklists);
        println!("Mobj spawning is done!");

        self.graphics.renderer = Some(renderer);
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
            WindowEvent::Resized(_) => {
                if let Some(renderer) = &mut self.graphics.renderer {
                    renderer.recreate_swapchain();
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat { return; }

                let is_pressed = event.state == ElementState::Pressed;

                if self.last_buttons_pressed[0] != Some(event.physical_key) {
                    self.last_buttons_pressed.pop_back();
                    self.last_buttons_pressed.push_front(Some(event.physical_key));
                }
                
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.current_input.move_forward = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.current_input.move_backward = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.current_input.move_left = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.current_input.move_right = is_pressed,
                    PhysicalKey::Code(KeyCode::Space) => self.current_input.move_up = is_pressed,
                    PhysicalKey::Code(KeyCode::ShiftLeft) => self.current_input.move_down = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit1) => self.current_input.switch_fist_chainsaw = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit2) => self.current_input.choose_pistol = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit3) => self.current_input.choose_shotgun = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit4) => self.current_input.choose_chaingun = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit5) => self.current_input.choose_rlauncher = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit6) => self.current_input.choose_plasma = is_pressed,
                    PhysicalKey::Code(KeyCode::Digit7) => self.current_input.choose_bfg = is_pressed,
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
            
                if let Err(err) = self.update_game_logic() {
                    handle_fatal_error(&mut self.is_shutting_down, event_loop, 
                        &mut self.graphics.renderer, &err.to_string());
                };
            
                if let Some(window) = &self.window {
                    let alpha = self.time_accumulator / TICK_TIME;
                    self.graphics.render(window, &self.game.world, self.game.player_entity, 
                        &self.game.level, self.game.state, self.game.global_timer, alpha);

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

fn handle_fatal_error(is_shutting_down: &mut bool, event_loop: &ActiveEventLoop, renderer: &mut Option<SafeRenderer>, msg: &str) {
    eprintln!("[FATAL] {}", msg);
    *is_shutting_down = true;
    if let Some(renderer) = renderer {
        renderer.shutdown();
    }
    
    event_loop.exit();
}

fn main() -> Result<(), String> {
    let mut wad_manager = WadManager::new();

    let args = Args::parse();

    wad_manager.add_wad(args.iwad)?;
    for pwad in args.pwads {
        wad_manager.add_wad(pwad)?;
    }

    let level = Level::load(&wad_manager, args.map)?;

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let flags = GraphicsFlags::new(args.wireframe, args.byte_shadows);

    let mut app = App {
        window: None,
        wad_manager,
        graphics: GraphicsContext::new(flags),
        game: GameContext::new(level, SkillLevel::from(args.skill_level), args.fast_monsters),
        audio: AudioContext::new()?,
        random: Random::default(),
        is_shutting_down: false,
        current_input: PlayerInput::default(),
        last_frame_time: Instant::now(),
        time_accumulator: 0.0,
        last_buttons_pressed: VecDeque::from_iter([None; 8]),
    };
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}

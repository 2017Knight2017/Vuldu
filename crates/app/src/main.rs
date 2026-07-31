mod prepare_for_renderer;
mod parse_commandline;
mod sound_player;

use rodio::MixerDeviceSink;
//use parse_commandline::Args;
use sound_player::*;
use renderer::*;
use wad_parser::map::DoomMap;
use wad_parser::*;
use engine::*;
use glam::Mat4;
use hecs::{CommandBuffer, Entity, World};
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::{WindowEvent, ElementState, DeviceEvent, DeviceId, MouseButton},
    window::{Window, WindowId, CursorGrabMode},
    keyboard::{PhysicalKey, KeyCode},
    dpi::LogicalSize
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
//use clap::Parser;
use rustc_hash::FxHashMap;
use std::time::Instant;
use std::f64::consts::TAU;

const EYE_HEIGHT: f32 = 41.0;
const FOV_ANGLE: f32 = 90.0;
const TICKRATE: u32 = 35;
const TICK_TIME: f32 = 1.0 / TICKRATE as f32;
pub const MAX_SKY: usize = 16;

struct App {
    window: Option<Window>,
    renderer: Option<SafeRenderer>,
    wad_manager: WadManager,
    world: World,
    map: DoomMap,
    dyn_map: DynMap,
    blocklists: Vec<Vec<Entity>>,
    random: Random,
    world_events: Vec<WorldEvent>,
    command_buffer: CommandBuffer,
    _audio_stream_handle: MixerDeviceSink,
    audio_player: DoomSfxPlayer,
    audio_buffer: Vec<SfxEvent>,
    texture_data: FxHashMap<u64, (u32, u32, u32, bool)>,
    audio_data: FxHashMap<u64, DoomSfx>,
    sprite_offsets: Vec<(i16, i16)>,
    is_shutting_down: bool,
    current_input: PlayerInput,
    view_matrix: Mat4,
    last_frame_time: Instant,
    time_accumulator: f32,
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

fn register_sprite(
    texture_data_map: &mut FxHashMap<u64, (u32, u32, u32, bool)>, 
    lump_name: &[u8], 
    texture_tuple: (u32, u32, u32)
) {
    let (id, w, h) = texture_tuple;

    let last_non_zero = lump_name.iter().rposition(|&b| b != 0).unwrap();
    let normed_name = &lump_name[..=last_non_zero];

    let prefix = &normed_name[..4];
    let frame1 = normed_name[4] as char;
    let view1 = normed_name[5] - b'0';

    let key1 = pack_sprite_u64(prefix, frame1, view1);
    texture_data_map.insert(key1, (id, w, h, false));

    if normed_name.len() == 8 {
        let frame2 = normed_name[6] as char;
        let view2 = normed_name[7] - b'0';

        let key2 = pack_sprite_u64(prefix, frame2, view2);
        texture_data_map.insert(key2, (id, w, h, true));
    }
}

fn update_camera_from_player(view_matrix: &mut Mat4, world: &World, alpha: f32) {
    for (position, rotation, _player) in world.query::<(&Position, &PlayerRotation, &PlayerMarker)>().iter() {

        let prev_pos = glam::vec3(position.prev_x, position.prev_y + EYE_HEIGHT, position.prev_z);
        let current_pos = glam::vec3(position.x, position.y + EYE_HEIGHT, position.z);
        let interpolated_pos = prev_pos + (current_pos - prev_pos) * alpha;

        let angle_diff = rotation.angle.wrapping_sub(rotation.prev_angle) as i32;
        let interpolated_diff = (angle_diff as f64 * alpha as f64) as i32;
        let interpolated_angle_u32 = rotation.prev_angle.wrapping_add_signed(interpolated_diff);

        let angle_normalized = interpolated_angle_u32 as f64 / u32::MAX as f64;
        let angle_rad = (angle_normalized * TAU) as f32;

        let target_dir = glam::vec3(f32::sin(angle_rad), 0.0, f32::cos(angle_rad));
        let camera_target = interpolated_pos + target_dir;

        let camera_up = glam::vec3(0.0, 1.0, 0.0);

        *view_matrix = glam::Mat4::look_at_rh(interpolated_pos, camera_target, camera_up);
    }
}

impl App {
    fn create_window(&self, event_loop: &ActiveEventLoop) -> Result<Window, Box<dyn std::error::Error>> {
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

    fn load_and_upload_textures(&mut self, renderer: &mut SafeRenderer) -> Result<(), String> {
        let (wall_names, wall_pics, sky_names, sky_pics, sky_widths) = 
            self.wad_manager.bake_walls().map_err(|e| format!("Wall baking failed: {e}"))?;

        let (flat_names, flat_pics) = 
            self.wad_manager.bake_flats().map_err(|e| format!("Flat baking failed: {e}"))?;

        let (obj_names, obj_pics) = 
            self.wad_manager.bake_objects().map_err(|e| format!("Object baking failed: {e}"))?;

        let total_textures = wall_pics.len() + flat_pics.len() + obj_pics.len() + MAX_SKY;
        let total_pixels = 1 + sky_pics.iter()
            .chain(&obj_pics)
            .chain(&wall_pics)
            .chain(&flat_pics)
            .map(|p| p.raw_pixels.len())
            .sum::<usize>();

        let mut all_pixels = Vec::with_capacity(total_pixels);
        let mut descriptors = Vec::with_capacity(total_textures);
        let mut current_gpu_id = 0;

        let mut sky_data: Vec<_> = sky_names.iter().zip(sky_pics).zip(sky_widths)
            .map(|((n, p), w)| (n, p, w)).collect();
        sky_data.sort_by_key(|trio| trio.0);
        current_gpu_id += MAX_SKY as u32;

        let mut sky_widths_no_name = Vec::with_capacity(sky_data.len());
        for (_, pic, width) in sky_data {
            descriptors.push(TextureDescriptor {
                width: pic.width,
                height: pic.height,
                pixel_offset: all_pixels.len(),
            });
            all_pixels.extend_from_slice(&pic.raw_pixels);
            sky_widths_no_name.push(width);
        }

        let padding_needed = MAX_SKY.saturating_sub(descriptors.len());                    
        for _ in 0..padding_needed {
            descriptors.push(TextureDescriptor {
                width: 1, height: 1, pixel_offset: all_pixels.len(),
            });
        }
        all_pixels.push(0);


        self.sprite_offsets.reserve(obj_pics.len());
        for (idx, pic) in obj_pics.iter().enumerate() {
            let name = obj_names[idx];
            self.sprite_offsets.push((pic.left_offset, pic.top_offset));
            register_sprite(&mut self.texture_data, name, (current_gpu_id, pic.width, pic.height));
            
            descriptors.push(TextureDescriptor {
                width: pic.width, height: pic.height, pixel_offset: all_pixels.len(),
            });
            all_pixels.extend_from_slice(&pic.raw_pixels);
            current_gpu_id += 1;
        }

        for (tex_names, pics) in [(&wall_names, &wall_pics), (&flat_names, &flat_pics)] {
            for (idx, pic) in pics.iter().enumerate() {
                let name = tex_names[idx];
                self.texture_data.insert(name, (current_gpu_id, pic.width, pic.height, false));
                descriptors.push(TextureDescriptor {
                    width: pic.width, height: pic.height, pixel_offset: all_pixels.len(),
                });
                all_pixels.extend_from_slice(&pic.raw_pixels);
                current_gpu_id += 1;
            }
        }

        renderer.upload_texture_array(&descriptors, &all_pixels, &sky_widths_no_name);

        let map_name = construct_map_name(self.wad_manager.is_doom1, self.map.map_num);
        let palettes = self.wad_manager.get_palettes(&map_name).map_err(|e| format!("PLAYPAL upload failed: {e}"))?;
        renderer.upload_palettes(&palettes);

        let colormap = self.wad_manager.get_colormap(&map_name).map_err(|e| format!("COLORMAP upload failed: {e}"))?;
        renderer.upload_colormap(colormap);

        Ok(())
    }

    fn setup_level_geometry(&mut self, renderer: &mut SafeRenderer) {
        println!("Building map geometry...");
        let (wall_vertices, wall_indices) = self.map.get_walls_vertices(&self.texture_data);
        println!("Walls geometry is has been built");
        let (flat_vertices, flat_indices) = self.map.get_flats_vertices(&self.texture_data);
        println!("Flats geometry is has been built");
        let (obj_vertices, obj_indices) = self.map.get_objects_vertices();

        let mut level_vertices = wall_vertices;
        let mut level_indices = wall_indices;

        let vertex_offset = level_vertices.len() as u32; 
        level_vertices.extend(flat_vertices);
        for idx in flat_indices {
            level_indices.push(vertex_offset + idx);
        }

        renderer.update_object_instances(&[]);
        renderer.update_level_geometry(&level_vertices, &level_indices);
        renderer.update_object_geometry(&obj_vertices, &obj_indices);
    }

    fn handle_fatal_error(&mut self, event_loop: &ActiveEventLoop, renderer: &mut SafeRenderer, msg: &str) {
        eprintln!("[FATAL] {}", msg);
        self.is_shutting_down = true;
        renderer.shutdown();
        event_loop.exit();
    }

    fn update_game_logic(&mut self) {
        while self.time_accumulator >= TICK_TIME {
            self.tick();
            self.time_accumulator -= TICK_TIME;
        }
    }

    fn tick(&mut self) {
        let ai_query = self.world.query::<&mut MobjAi>();
        ai_system(ai_query);

        let position_input_query = self.world.query::<(Entity, &mut Velocity, &PlayerRotation)>();
        let animation_query = self.world.query::<(&mut SpriteAnimation, &MobjAi)>();
        micropool::join(
            || handle_position_input(position_input_query, &self.current_input, &mut self.command_buffer, &mut self.audio_buffer),
            || animation_system(animation_query),
        );

        self.flush_command_buffer();

        let rotation_query = self.world.query::<&mut PlayerRotation>();
        let friction_query = self.world.query::<&mut Velocity>();
        micropool::join(
            || handle_rotation_input(rotation_query, &self.current_input),
            || friction_system(friction_query),
        );

        let propagate_sound_query = self.world.query::<(Entity, &CurrentSector)>().with::<&PlayerShoot>();
        propagate_sound_system(
            propagate_sound_query, 
            &mut self.command_buffer, 
            &mut self.dyn_map.sectors, 
            &self.map.linedefs, 
            &self.map.sidedefs
        );

        self.flush_command_buffer();

        let check_sound_query = self.world
            .query::<(Entity, &mut SpriteAnimation, &mut MobjAi, &Position, &CurrentSector, &MobjType)>()
            .with::<&Idle>();
        check_sound_system(
            check_sound_query, 
            &self.world, 
            &self.map, 
            &mut self.dyn_map,
            &mut self.random, 
            &mut self.command_buffer, 
            &mut self.audio_buffer
        );

        //let check_sight_query = self.world
        //    .query::<(Entity, &mut SpriteAnimation, &mut MobjAi, &Position, &CurrentSector, &MonsterRotation, &MobjType)>()
        //    .with::<&Idle>();
        //let players_query = self.world
        //    .query::<(Entity, &Position, &CurrentSector, &MobjType)>()
        //    .with::<&PlayerMarker>();
        //check_sight_system(
        //    check_sight_query, 
        //    players_query, 
        //    &self.map, 
        //    &mut self.dyn_map,
        //    &mut self.random, 
        //    &mut self.command_buffer, 
        //    &mut self.audio_buffer
        //);

        self.flush_command_buffer();

        let chase_query = self.world.query::<(Entity, &mut MonsterRotation, &mut MobjType, &mut MobjAi, &mut InstantMoveIntent, &Position, &Target)>();
        chase_system(
            chase_query, 
            &self.world, 
            &mut self.random, 
            &self.map, 
            SkillLevel::Hard, 
            true, 
            &mut self.audio_buffer, 
            &self.blocklists, 
            &mut self.world_events
        );

        let monster_movement_query = self.world
            .query::<(Entity, &mut Position, &mut CurrentSector, &mut InstantMoveIntent, &mut Velocity, &MobjType)>()
            .with::<&Active>();
        monster_movement_system(
            monster_movement_query, 
            &self.map,
            &self.world,
            &mut self.random,
            &mut self.blocklists,
            &mut self.world_events
        );

        let player_movement_query = self.world
            .query::<(&mut Position, &Velocity, &mut CurrentSector)>()
            .with::<&PlayerMarker>();
        player_movement_system(player_movement_query, &self.map);

        let audio_query = self.world.query::<(&Position, &PlayerRotation)>();
        micropool::join(
            || audio_system(audio_query, &mut self.audio_buffer, &mut self.audio_player, &self.audio_data),
            || execute_events_system(&mut self.world_events)
        );

        self.current_input.mouse_delta_x = 0.0;
    }

    fn flush_command_buffer(&mut self) {
        self.command_buffer.run_on(&mut self.world);
        self.command_buffer.clear();
    }

    fn render(&mut self, alpha: f32) {
        let instances = self.collect_object_instances(alpha);

        let (renderer, window) = match (&mut self.renderer, &self.window) {
            (Some(r), Some(w)) => (r, w),
            _ => return,
        };

        renderer.update_object_instances(&instances);
        
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let aspect_ratio = size.width as f32 / size.height as f32;
        let proj = Mat4::perspective_rh(FOV_ANGLE.to_radians(), aspect_ratio, 1.0, 10000.0);

        update_camera_from_player(&mut self.view_matrix, &self.world, alpha);

        let ubo = UniformBufferObject {
            model: Mat4::IDENTITY.to_cols_array(),
            view: self.view_matrix.to_cols_array(),
            proj: proj.to_cols_array(),
        };

        renderer.start_frame(&ubo);
        renderer.draw_level();
        renderer.draw_objects();
        renderer.end_frame();
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

        for (line_idx, line) in self.map.linedefs.iter().enumerate() {
		    if line.sidenum[0] != u16::MAX {
		        let front_sector = self.map.sidedefs[line.sidenum[0] as usize].sector;
		        self.dyn_map.sectors[front_sector as usize].lines.push(line_idx);
		    }
		    if line.sidenum[1] != u16::MAX {
		        let back_sector = self.map.sidedefs[line.sidenum[1] as usize].sector;
		        self.dyn_map.sectors[back_sector as usize].lines.push(line_idx);
		    }
		}

        self.audio_data = self.wad_manager.bake_sfx();
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
                (self.map.map_num as u32 - 1) / 9 
            } else { 
                get_sky_texture_index(self.map.map_num) 
            };
            renderer.set_sky_index(sky_idx);

            if let Err(err) = self.load_and_upload_textures(&mut renderer) {
                self.handle_fatal_error(event_loop, &mut renderer, &err);
                return;
            }

            self.setup_level_geometry(&mut renderer);

            let _ = engine::populate_database(&self.texture_data).map_err(|e| eprintln!("{}", e));
            engine::spawn_all_things(&mut self.world, &self.map, &mut self.random);
            println!("Mobj spawning is done!");

            self.renderer = Some(renderer);
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
                if let Some(renderer) = &mut self.renderer {
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
            },

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
                if let Some(renderer) = &mut self.renderer {
                    renderer.shutdown();
                }
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if self.is_shutting_down { 
                    return; 
                }
            
                let current_time = std::time::Instant::now();
                let delta_time = current_time.duration_since(self.last_frame_time).as_secs_f32().min(0.25);
                self.last_frame_time = current_time;
                self.time_accumulator += delta_time;
            
                self.update_game_logic();
            
                let alpha = self.time_accumulator / TICK_TIME;
                self.render(alpha);
            
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
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
    //let map = DoomMap::from_wad(&wad_manager, &args.map)?;

    wad_manager.add_wad("assets/DOOM.WAD")?;
    //wad_manager.add_wad("assets/oku2v31.wad")?;
    //wad_manager.add_wad("assets/nuts.wad")?;
    //wad_manager.add_wad("assets/Sunder 2512.wad")?;
    //wad_manager.add_wad("assets/HR.WAD")?;

    let map = DoomMap::from_wad(&wad_manager, 1)?;
    let dyn_map = DynMap::from(&map);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut audio_stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
        .map_err(|_| "Failed to create an audio stream handle".to_string())?;
    audio_stream_handle.log_on_drop(false);
    let audio_player = DoomSfxPlayer::new(&audio_stream_handle);

    let mut app = App {
        window: None,
        renderer: None,
        wad_manager: wad_manager,
        world: World::new(),
        blocklists: vec![Vec::new(); map.blockmap.row_num * map.blockmap.col_num],
        map,
        dyn_map,
        random: Random::default(),
        world_events: Vec::new(),
        _audio_stream_handle: audio_stream_handle,
        audio_player,
        audio_buffer: Vec::new(),
        command_buffer: CommandBuffer::new(),
        texture_data: FxHashMap::default(),
        audio_data: FxHashMap::default(),
        sprite_offsets: Vec::new(),
        is_shutting_down: false,
        current_input: PlayerInput::default(),
        view_matrix: Mat4::default(),
        last_frame_time: Instant::now(),
        time_accumulator: 0.0,
    };
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}

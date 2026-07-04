mod prepare_for_renderer;

use renderer::*;
use wad_parser::map::DoomMap;
use wad_parser::*;
use engine::*;
use glam::Mat4;
use hecs::World;
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::{WindowEvent, ElementState, DeviceEvent, DeviceId},
    window::{Window, WindowId, CursorGrabMode},
    keyboard::{PhysicalKey, KeyCode}
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::time::Instant;
use std::collections::HashMap;

const EYE_HEIGHT: f32 = 41.0;
const FOV_ANGLE: f32 = 90.0;
const TICKRATE: u32 = 35;
const TICK_TIME: f32 = 1.0 / TICKRATE as f32;

struct App {
    window: Option<Window>,
    renderer: Option<SafeRenderer>,
    wad: Wad,
    world: World,
    map: DoomMap,
    texture_data: HashMap<String, (u32, u32, u32)>,
    sprite_offsets: Vec<(i16, i16)>,
    is_shutting_down: bool,
    current_input: PlayerInput,
    view_matrix: Mat4,
    last_frame_time: Instant,
    time_accumulator: f32,
}

fn register_sprite(texture_data_map: &mut HashMap<String, (u32, u32, u32)>, lump_name: &str, texture_tuple: (u32, u32, u32)) {
    if lump_name.len() == 6 {
        texture_data_map.insert(lump_name.to_string(), texture_tuple);
    } else if lump_name.len() == 8 {
        let prefix = &lump_name[0..4]; 

        let frame1 = lump_name.chars().nth(4).unwrap(); 
        let view1  = lump_name.chars().nth(5).unwrap(); 

        let frame2 = lump_name.chars().nth(6).unwrap(); 
        let view2  = lump_name.chars().nth(7).unwrap(); 

        texture_data_map.insert(format!("{}{}{}", prefix, frame1, view1), texture_tuple); // VILEA1
        texture_data_map.insert(format!("{}{}{}_FLIP", prefix, frame2, view2), texture_tuple); // VILED1
    }
}

impl App {
    fn update_camera_from_player(&mut self, alpha: f32) {
        for (transform, _player) in self.world.query::<(&Transform, &PlayerMarker)>().iter() {

            let prev_pos = glam::vec3(transform.prev_x, transform.prev_y + EYE_HEIGHT, transform.prev_z);
            let current_pos = glam::vec3(transform.x, transform.y + EYE_HEIGHT, transform.z);
            let interpolated_pos = prev_pos + (current_pos - prev_pos) * alpha;

            let angle_diff = transform.angle.wrapping_sub(transform.prev_angle) as i32;
            let interpolated_diff = (angle_diff as f64 * alpha as f64) as i32;
            let interpolated_angle_u32 = transform.prev_angle.wrapping_add_signed(interpolated_diff);

            let angle_normalized = interpolated_angle_u32 as f64 / u32::MAX as f64;
            let angle_rad = (angle_normalized * std::f64::consts::TAU) as f32;

            let target_dir = glam::vec3(f32::sin(angle_rad), 0.0, f32::cos(angle_rad));
            let camera_target = interpolated_pos + target_dir;

            let camera_up = glam::vec3(0.0, 1.0, 0.0);

            self.view_matrix = glam::Mat4::look_at_rh(interpolated_pos, camera_target, camera_up);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop.create_window(Window::default_attributes()).unwrap();
            
            window.set_cursor_visible(false);

            if let Err(err) = window.set_cursor_grab(CursorGrabMode::Locked) {
                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                println!("{:?}", err);
            }

            self.window = Some(window);

            let _ = engine::populate_database().map_err(|err| println!("{}", err));

            let plr_spawn = self.map.things
                .iter().find(|thing| thing.type_ == 1)
                .unwrap();
            let plr_floorheight = self.map.sectors[self.map.get_sector_by_pos(plr_spawn.x as f32, plr_spawn.y as f32)].floorheight;
            engine::spawn_mobj(&mut self.world, Some(MobjType::Player), -plr_spawn.x, plr_floorheight, plr_spawn.y, plr_spawn.angle);

            engine::spawn_all_things(&mut self.world, &self.map);
            
            self.renderer = Some(SafeRenderer::new());

            let window_handle = self.window.as_ref().unwrap().window_handle().unwrap().as_raw();
            let display_handle = self.window.as_ref().unwrap().display_handle().unwrap().as_raw();

            if let (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) = (display_handle, window_handle) {
                let handles = WindowHandles {
                    display_ptr: d.display.as_ptr() as usize,
                    window_ptr: w.surface.as_ptr() as usize,
                };

                if let Some(renderer) = &mut self.renderer {
                    let window_raw_ptr = self.window.as_ref().unwrap() as *const Window as usize;

                    renderer.init(&handles, window_raw_ptr);

                    let (wall_texture_names, wall_pics) = self.wad.bake_walls().unwrap();
                    let (flat_texture_names, flat_pics) = self.wad.bake_flats().unwrap(); 
                    let (obj_texture_names, obj_pics) = self.wad.bake_objects().unwrap(); 

                    let total_textures_count = wall_pics.len() + flat_pics.len() + obj_pics.len();
                    let mut all_pixels = Vec::new();
                    let mut descriptors = Vec::with_capacity(total_textures_count);

                    let mut current_gpu_id = 0;

                    let mut are_objects_recording = true;

                    for (tex_names, pics) in [
                        (obj_texture_names, obj_pics),
                        (wall_texture_names, wall_pics),
                        (flat_texture_names, flat_pics)].iter() 
                    {
                        for (idx, pic) in pics.iter().enumerate() {
                            let name = &tex_names[idx];

                            if are_objects_recording {
                                self.sprite_offsets.push((pic.left_offset, pic.top_offset));
                                register_sprite(&mut self.texture_data, name, (current_gpu_id, pic.width, pic.height))
                            } else {
                                self.texture_data.insert(name.clone(), (current_gpu_id, pic.width, pic.height));
                            }
                        
                            descriptors.push(TextureDescriptor {
                                width: pic.width,
                                height: pic.height,
                                pixel_offset: all_pixels.len(),
                            });

                            for &lump_pixel in &pic.raw_pixels {
                                all_pixels.push(lump_pixel);
                            }
                        
                            current_gpu_id += 1;
                        }
                        are_objects_recording = false;
                    }

                    renderer.upload_texture_array(descriptors.as_slice(), all_pixels.as_slice());

                    renderer.upload_palettes(self.wad
                        .get_palettes()
                        .expect("No PLAYPAL in the wad!")
                        .as_slice()
                    );
                    renderer.upload_colormap(self.wad
                        .get_data_by_lumpname("COLORMAP")
                        .expect("No COLORMAP in the wad!")
                    );

                    let (wall_vertices, wall_indices) = self.map.get_walls_vertices(&self.texture_data);
                    let (flat_vertices, flat_indices) = self.map.get_flats_vertices(&self.texture_data);
                    let (obj_vertices, obj_indices) = self.map.get_objects_vertices();

                    let mut level_vertices = wall_vertices;
                    let mut level_indices = wall_indices;

                    let vertex_offset = level_vertices.len() as u16; 
                    level_vertices.extend(flat_vertices);

                    for idx in flat_indices {
                        level_indices.push(vertex_offset + idx);
                    }

                    renderer.update_object_instances(&[]);
                    renderer.update_level_geometry(&level_vertices, &level_indices);
                    renderer.update_object_geometry(&obj_vertices, &obj_indices);
                }
            }
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
            }

            _ => {}
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
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

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                self.is_shutting_down = true;
                if let Some(renderer) = &mut self.renderer {
                    renderer.shutdown();
                }
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if self.is_shutting_down { return; }

                let current_time = std::time::Instant::now();
                let delta_time = current_time.duration_since(self.last_frame_time).as_secs_f32();
                self.last_frame_time = current_time;

                let delta_time = delta_time.min(0.25); 
                self.time_accumulator += delta_time;

                while self.time_accumulator >= TICK_TIME {
                    
                    engine::update_physics(&mut self.world, &self.current_input);
                    engine::system_movement_and_friction(&mut self.world);

                    engine::animation_system(&mut self.world);
                    
                    self.current_input.mouse_delta_x = 0.0;
                    self.time_accumulator -= TICK_TIME;
                }

                let alpha = self.time_accumulator / TICK_TIME;
                self.update_camera_from_player(alpha);

                let instances = self.collect_object_instances(alpha);

                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }

                    renderer.update_object_instances(&instances);

                    let aspect_ratio = size.width as f32 / size.height as f32;
                
                    let proj = Mat4::perspective_rh(FOV_ANGLE.to_radians(), aspect_ratio, 1.0, 10000.0);
                
                    let ubo = UniformBufferObject {
                        model: Mat4::IDENTITY.to_cols_array(),
                        view: self.view_matrix.to_cols_array(),
                        proj: proj.to_cols_array(),
                    };
                    
                    renderer.start_frame(&ubo);
                    renderer.draw_level();
                    renderer.draw_objects();
                    renderer.end_frame();
                
                    renderer.set_palette_index(0);
                
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            },

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
    let wad = Wad::open("assets/DOOM2.WAD")?;
    let map = DoomMap::from_wad(&wad, "MAP24")?;

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        renderer: None,
        wad: wad,
        world: World::new(),
        map: map,
        texture_data: HashMap::new(),
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

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
    window::{Window, WindowId},
    keyboard::{PhysicalKey, KeyCode}
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::time::Instant;
use std::collections::HashMap;

struct App {
    window: Option<Window>,
    renderer: Option<SafeRenderer>,
    wad: Wad,
    world: World,
    map: DoomMap,
    start_time: Instant,
    is_shutting_down: bool,
    current_input: PlayerInput,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop.create_window(Window::default_attributes()).unwrap();
            self.window = Some(window);
            
            self.renderer = Some(SafeRenderer::new());

            let window_handle = self.window.as_ref().unwrap().window_handle().unwrap().as_raw();
            let display_handle = self.window.as_ref().unwrap().display_handle().unwrap().as_raw();

            if let (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) = (display_handle, window_handle) {
                let handles = WindowHandles {
                    display_ptr: d.display.as_ptr() as usize,
                    window_ptr: w.surface.as_ptr() as usize,
                };

                if let Some(renderer) = &mut self.renderer {
                    let window_raw_ptr = self.window.as_ref().unwrap() as *const winit::window::Window as usize;

                    renderer.init(&handles, window_raw_ptr);

                    let (wall_texture_names, wall_pics) = self.wad.bake_walls().unwrap();
                    let (flat_texture_names, flat_pics) = self.wad.bake_flats().unwrap(); 
                    let (obj_texture_names, obj_pics) = self.wad.bake_objects().unwrap(); 

                    let total_textures_count = wall_pics.len() + flat_pics.len() + obj_pics.len();
                    let mut all_pixels = Vec::new();
                    let mut descriptors = Vec::with_capacity(total_textures_count);

                    let mut texture_data = HashMap::new();
                    let mut current_gpu_id = 0;

                    let mut sprite_offsets = Vec::new();
                    let mut are_objects_recording = true;

                    for (tex_names, pics) in [
                        (obj_texture_names, obj_pics),
                        (wall_texture_names, wall_pics),
                        (flat_texture_names, flat_pics)].iter() 
                    {
                        for (idx, pic) in pics.iter().enumerate() {
                            let name = &tex_names[idx];
                        
                            descriptors.push(TextureDescriptor {
                                width: pic.width,
                                height: pic.height,
                                pixel_offset: all_pixels.len(),
                            });

                            for &lump_pixel in &pic.raw_pixels {
                                all_pixels.push(lump_pixel);
                                all_pixels.push(0);
                                all_pixels.push(0);
                                all_pixels.push(255);
                            }
                        
                            texture_data.insert(name.clone(), (current_gpu_id, pic.width, pic.height));
                            current_gpu_id += 1;

                            if are_objects_recording {
                                sprite_offsets.push((pic.left_offset, pic.top_offset));
                            }
                        }
                        are_objects_recording = false;
                    }

                    renderer.upload_texture_array(descriptors.as_slice(), descriptors.len(), all_pixels.as_slice(), all_pixels.len());

                    renderer.upload_palettes(self.wad
                        .get_palettes()
                        .expect("No PLAYPAL in the wad!")
                        .as_slice()
                    );
                    renderer.upload_colormap(self.wad
                        .get_data_by_lumpname("COLORMAP")
                        .expect("No COLORMAP in the wad!")
                    );

                    let (wall_vertices, wall_indices) = self.map.get_walls_vertices(&texture_data);
                    let (flat_vertices, flat_indices) = self.map.get_flats_vertices(&texture_data);
                    let (obj_vertices, obj_indices) = self.map.get_objects_vertices(&texture_data, sprite_offsets);

                    let mut level_vertices = wall_vertices;
                    let mut level_indices = wall_indices;

                    let vertex_offset = level_vertices.len() as u16; 
                    level_vertices.extend(flat_vertices);

                    for idx in flat_indices {
                        level_indices.push(vertex_offset + idx);
                    }

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
                let is_pressed = event.state == ElementState::Pressed;
                
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.current_input.move_forward = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.current_input.move_backward = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.current_input.move_left = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.current_input.move_right = is_pressed,
                    PhysicalKey::Code(KeyCode::KeyQ) => self.current_input.shoot = is_pressed,
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

                engine::update_physics(&mut self.world, &self.current_input);
                self.current_input.mouse_delta_x = 0.0;

                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }

                    let aspect_ratio = size.width as f32 / size.height as f32;
                    let time = self.start_time.elapsed().as_secs_f32();
                
                    let model = Mat4::IDENTITY; 

                    let player1_spawner = self.map.things
                        .iter().find(|thing| thing.type_ == 1)
                        .unwrap();

                    let player_x = -player1_spawner.x as f32 - 250.0;
                    let player_y = 90.0;
                    let player_z = player1_spawner.y as f32 + 1900.0;
                    
                    let camera_pos = glam::vec3(player_x - time*10.0, player_y+time*6.0, player_z - time*60.0);
                    let camera_target = glam::vec3(player_x - time*10.1 + f32::sin(time/3.0)*10.0, player_y+time*5.8, player_z-10.0 - time*60.0);
                    let camera_up = glam::vec3(0.0, 1.0, 0.0);
                
                    let view = Mat4::look_at_rh(camera_pos, camera_target, camera_up);
                
                    let proj = Mat4::perspective_rh(90.0f32.to_radians(), aspect_ratio, 1.0, 10000.0);
                    //proj.col_mut(1)[1] *= -1.0;
                
                    let ubo = UniformBufferObject {
                        model: model.to_cols_array(),
                        view: view.to_cols_array(),
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
    let map = DoomMap::from_wad(&wad, "MAP01")?;

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        renderer: None,
        wad: wad,
        world: World::new(),
        map: map,
        start_time: Instant::now(),
        is_shutting_down: false,
        current_input: PlayerInput::default()
    };
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
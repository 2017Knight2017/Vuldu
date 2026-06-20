use renderer::*;
use wad_parser::map::DoomMap;
use wad_parser::*;
use glam::Mat4;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::time::Instant;
use std::collections::HashMap;

struct SpriteFrame {
    texture_id: u32,
    width: u32,
    height: u32,
    left_offset: i16,
    top_offset: i16,
}

struct App {
    window: Option<Window>,
    renderer: Option<SafeRenderer>,
    wad: Wad,
    map: DoomMap,
    textures: Vec<SpriteFrame>,
    start_time: Instant,
    is_shutting_down: bool
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

                    let total_textures_count = wall_pics.len() + flat_pics.len();
                    let mut all_pixels = Vec::new();
                    let mut descriptors = Vec::with_capacity(total_textures_count);

                    let mut texture_data = HashMap::new();
                    let mut current_gpu_id = 0;

                    for (idx, pic) in wall_pics.iter().enumerate() {
                        let name = &wall_texture_names[idx];
                    
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
                    }
                
                    for (idx, pic) in flat_pics.iter().enumerate() {
                        let name = &flat_texture_names[idx];

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

                    let mut all_vertices = wall_vertices;
                    let mut all_indices = wall_indices;

                    let vertex_offset = all_vertices.len() as u32; 

                    all_vertices.extend(flat_vertices);

                    for index in flat_indices {
                        all_indices.push((vertex_offset + index as u32) as u16);
                    }

                    renderer.update_geometry(&all_vertices, &all_indices);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                self.is_shutting_down = true;
                if let Some(renderer) = &mut self.renderer {
                    renderer.shutdown();
                }
                event_loop.exit();
            },

            WindowEvent::RedrawRequested => {
                if self.is_shutting_down { return; }
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

                    let player_x = -player1_spawner.x as f32;
                    let player_y = 90.0;
                    let player_z = player1_spawner.y as f32 - 30.0;
                    
                    let camera_pos = glam::vec3(player_x - time*10.0, player_y, player_z + time*50.0);
                    let camera_target = glam::vec3(player_x - time*20.0, player_y, player_z+100.0 + time*50.0);
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
        map: map,
        textures: Vec::new(),
        start_time: Instant::now(),
        is_shutting_down: false
    };
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
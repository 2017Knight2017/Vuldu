use renderer::ffi;
use wad_parser::Wad;
use wad_parser::sprite::Sprite;
use glam::{Mat4, vec3};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use cxx::UniquePtr;
use std::time::Instant;

struct Entity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub health: i32,
    pub texture_id: u32,
    pub width: u32,
    pub height: u32,
    pub left_offset: i16,
    pub top_offset: i16,
    pub current_sprite: String,
}

struct SpriteFrame {
    texture_id: u32,
    width: u32,
    height: u32,
    left_offset: i16,
    top_offset: i16,
}

struct App {
    window: Option<Window>,
    renderer: Option<UniquePtr<ffi::VulkanRenderer>>,
    wad: Wad,
    obj: Vec<Entity>,
    textures: Vec<SpriteFrame>,
    start_time: Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop.create_window(Window::default_attributes()).unwrap();
            self.window = Some(window);
            
            let cpp_renderer = ffi::createRenderer();
            self.renderer = Some(cpp_renderer);

            let window_handle = self.window.as_ref().unwrap().window_handle().unwrap().as_raw();
            let display_handle = self.window.as_ref().unwrap().display_handle().unwrap().as_raw();

            if let (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) = (display_handle, window_handle) {
                let handles = ffi::WindowHandles {
                    display_ptr: d.display.as_ptr() as usize,
                    window_ptr: w.surface.as_ptr() as usize,
                };

                if let Some(renderer) = self.renderer.as_mut() {
                    let window_raw_ptr = self.window.as_ref().unwrap() as *const winit::window::Window as usize;

                    renderer.pin_mut().initVulkan(&handles, window_raw_ptr);

                    let mut all_palettes_data = vec![0.0f32; 14 * 256 * 4];
                    
                    let playpal_lump = self.wad.get_data_by_lumpname("PLAYPAL").expect("No palette!");

                    for palette_idx in 0..14 {
                        for color_idx in 0..256 {
                            let global_color_offset = palette_idx * 256 * 3 + color_idx * 3;
                            let target_offset = (palette_idx * 256 + color_idx) * 4;
                        
                            all_palettes_data[target_offset + 0] = playpal_lump[global_color_offset + 0] as f32 / 255.0;
                            all_palettes_data[target_offset + 1] = playpal_lump[global_color_offset + 1] as f32 / 255.0;
                            all_palettes_data[target_offset + 2] = playpal_lump[global_color_offset + 2] as f32 / 255.0;
                            all_palettes_data[target_offset + 3] = 1.0; 
                        }
                    }

                    unsafe {
                        renderer.pin_mut().uploadPalettes(all_palettes_data.as_ptr());
                    }

                    let names = ["TROOA1", "TROOB1", "TROOC1", "TROOD1"];
                    let mut frames = Vec::new();

                    for name in names {
                        let pic = self.wad
                            .into_raw_pixels(self.wad.directory.get(name).cloned().expect("Lump missing"))
                            .unwrap();

                        let tex_id = unsafe {
                            renderer.pin_mut().addTexture(pic.raw_pixels.as_ptr(), pic.width, pic.height)
                        };

                        frames.push(SpriteFrame {
                            texture_id: tex_id,
                            width: pic.width,
                            height: pic.height,
                            left_offset: pic.left_offset,
                            top_offset: pic.top_offset,
                        });
                    }
                    self.textures = frames;


                    let mesh_vertices = vec![
                        ffi::Vertex::new(glam::vec3(-0.5, -0.5, 0.0), glam::vec3(1.0, 0.0, 0.0), glam::vec2(1.0, 0.0), self.obj[0].texture_id), 
                        ffi::Vertex::new(glam::vec3(0.5, -0.5, 0.0),  glam::vec3(0.0, 1.0, 0.0), glam::vec2(0.0, 0.0), self.obj[0].texture_id), 
                        ffi::Vertex::new(glam::vec3(0.5, 0.5, 0.0),  glam::vec3(0.0, 0.0, 1.0), glam::vec2(0.0, 1.0), self.obj[0].texture_id), 
                        ffi::Vertex::new(glam::vec3(-0.5, 0.5, 0.0), glam::vec3(1.0, 1.0, 1.0), glam::vec2(1.0, 1.0), self.obj[0].texture_id),  
                    ];

                    let mesh_indices: Vec<u16> = Vec::from([
                        0, 1, 2, 2, 3, 0,
                    ]);

                    unsafe {
                        renderer.pin_mut().updateGeometry(mesh_vertices.as_ptr(), mesh_vertices.len(), mesh_indices.as_ptr(), mesh_indices.len());
                    }
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                if let Some(mut renderer) = self.renderer.take() {
                    renderer.pin_mut().cleanup();
                }
                event_loop.exit();
            },

            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(window)) = (self.renderer.as_mut(), &self.window) {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }
                    
                    let aspect_ratio = size.width as f32 / size.height as f32;
                    let time = self.start_time.elapsed().as_secs_f32();
                
                    let model = Mat4::from_rotation_z(time * 90.0f32.to_radians());

                    let view = Mat4::look_at_rh(
                        glam::vec3(0.0, -5.0 + time / 3.0, 1.0),
                        glam::vec3(0.0, 0.0, 1.0),   
                        glam::vec3(0.0, 0.0, 1.0)    
                    );

                    let proj = Mat4::perspective_rh(45.0f32.to_radians(), aspect_ratio, 0.1, 100.0);
                
                    let ubo = ffi::UniformBufferObject {
                        model: model.to_cols_array(),
                        view: view.to_cols_array(),
                        proj: proj.to_cols_array(),
                    };
                
                    unsafe { renderer.pin_mut().startFrame(&ubo); } 

                        //renderer.pin_mut().drawLevel();

                    let frame_idx = ((time * 5.0) as usize) % 4; 
                    let current_frame = &self.textures[frame_idx];

                    for imp in &self.obj {
                        renderer.pin_mut().drawSprite(
                            current_frame.texture_id,
                            current_frame.width, 
                            current_frame.height,
                            current_frame.left_offset, 
                            current_frame.top_offset,
                            imp.x, imp.y, imp.z
                        );
                    }
                    
                    renderer.pin_mut().endFrame();

                    renderer.pin_mut().setPaletteIndex(time as u32 / 2);

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
    let mut obj: Vec<Entity> = Vec::new();
    obj.push(Entity { 
        x: 0.0, y: 0.0, z: 1.0,
        health: 0, texture_id: 0, width: 0, height: 0,
        left_offset: 0, top_offset: 0, current_sprite: "TROOA1".to_string()
    });

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        renderer: None,
        wad: wad,
        obj: obj,
        textures: Vec::new(),
        start_time: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
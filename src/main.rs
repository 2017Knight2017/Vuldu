#[allow(non_snake_case)]
mod FFI;

use FFI::ffi;
use glam::{Mat4, vec3};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use cxx::UniquePtr;
use std::time::Instant;

struct App {
    window: Option<Window>,
    renderer: Option<UniquePtr<ffi::VulkanRenderer>>,
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

            let img = image::open("images.jpg")
                .expect("No textures found at images.jpg!")
                .to_rgba8();
            let (image_width, image_height) = img.dimensions();
            let raw_img = img.as_raw();

            if let (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) = (display_handle, window_handle) {
                let handles = ffi::WindowHandles {
                    display_ptr: d.display.as_ptr() as usize,
                    window_ptr: w.surface.as_ptr() as usize,
                };

                if let Some(renderer) = self.renderer.as_mut() {
                    let window_raw_ptr = self.window.as_ref().unwrap() as *const winit::window::Window as usize;

                    renderer.pin_mut().initVulkan(&handles, window_raw_ptr);

                    let mesh_vertices = vec![
                        ffi::Vertex::new(glam::vec2(-0.5, -0.5), glam::vec3(1.0, 0.0, 0.0), glam::vec2(1.0, 0.0)), 
                        ffi::Vertex::new(glam::vec2(0.5, -0.5),  glam::vec3(0.0, 1.0, 0.0), glam::vec2(0.0, 0.0)), 
                        ffi::Vertex::new(glam::vec2(0.5, 0.5),  glam::vec3(0.0, 0.0, 1.0), glam::vec2(0.0, 1.0)), 
                        ffi::Vertex::new(glam::vec2(-0.5, 0.5), glam::vec3(1.0, 1.0, 1.0), glam::vec2(1.0, 1.0)),
                    ];

                    unsafe {
                        renderer.pin_mut().setVertices(mesh_vertices.as_ptr(), mesh_vertices.len());
                        renderer.pin_mut().setTexture(raw_img.as_ptr(), image_width, image_height);
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
                    let view  = Mat4::look_at_rh(vec3(2.0, 2.0, 2.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
                    let mut proj = Mat4::perspective_rh(45.0f32.to_radians(), aspect_ratio, 0.1, 10.0);
                
                    proj.col_mut(1).y *= -1.0; 
                
                    let ubo = ffi::UniformBufferObject {
                        model: model.to_cols_array(),
                        view: view.to_cols_array(),
                        proj: proj.to_cols_array(),
                    };
                
                    unsafe { renderer.pin_mut().drawFrame(&ubo); }

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

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        renderer: None,
        start_time: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();
}
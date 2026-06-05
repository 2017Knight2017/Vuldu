use glam;

#[cxx::bridge]
pub mod ffi {
    struct WindowHandles {
        display_ptr: usize, 
        window_ptr: usize, 
    }

    pub struct WindowSize {
        pub width: u32,
        pub height: u32,
    }

    pub struct Vertex {
        pub pos: [f32; 2],
        pub color: [f32; 3],
    }

    pub struct UniformBufferObject {
        pub model: [f32; 16],
        pub view: [f32; 16],
        pub proj: [f32; 16],
    }

    extern "Rust" {
        unsafe fn get_winit_window_size(window_raw_ptr: usize) -> WindowSize;
    }

    unsafe extern "C++" {
        include!("renderer.h");

        type VulkanRenderer;

        fn createRenderer() -> UniquePtr<VulkanRenderer>;
        fn initVulkan(self: Pin<&mut VulkanRenderer>, handles: &WindowHandles, window_raw_ptr: usize);
        fn cleanup(self: Pin<&mut VulkanRenderer>);
        unsafe fn drawFrame(self: Pin<&mut VulkanRenderer>, ubo_ptr: *const UniformBufferObject);
        unsafe fn setVertices(self: Pin<&mut VulkanRenderer>, vertices_ptr: *const Vertex, count: usize);
    }
}

unsafe fn get_winit_window_size(window_raw_ptr: usize) -> ffi::WindowSize {
    let window_ref = unsafe { &*(window_raw_ptr as *const winit::window::Window) };
    
    let size = window_ref.inner_size();
    ffi::WindowSize {
        width: size.width,
        height: size.height,
    }
}

impl ffi::Vertex {
    pub fn new(pos: glam::Vec2, color: glam::Vec3) -> ffi::Vertex {
        ffi::Vertex {
            pos: pos.to_array(),
            color: color.to_array(),
        }
    }
}
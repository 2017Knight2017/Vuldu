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
        pub pos: [f32; 3],
        pub color: [f32; 3],
        pub texture_pos: [f32; 2],
        pub texture_id: i32,
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
        include!("utils.h");

        type VulkanRenderer;

        fn createRenderer() -> UniquePtr<VulkanRenderer>;
        fn initVulkan(self: Pin<&mut VulkanRenderer>, handles: &WindowHandles, window_raw_ptr: usize);
        fn cleanup(self: Pin<&mut VulkanRenderer>);
        unsafe fn drawFrame(self: Pin<&mut VulkanRenderer>, ubo_ptr: *const UniformBufferObject);
        unsafe fn addTexture(self: Pin<&mut VulkanRenderer>, pixels: *const u8, width: u32, height: u32) -> i32;
        unsafe fn updateGeometry(self: Pin<&mut VulkanRenderer>, vertices: *const Vertex, vertex_count: usize, indices: *const u16, index_count: usize);
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
    pub fn new(pos: glam::Vec3, color: glam::Vec3, texture_pos: glam::Vec2, texture_id: i32) -> ffi::Vertex {
        ffi::Vertex {
            pos: pos.to_array(),
            color: color.to_array(),
            texture_pos: texture_pos.to_array(),
            texture_id: texture_id,
        }
    }
}
#[cxx::bridge]
pub(crate) mod ffi {
    pub struct WindowHandles {
        display_ptr: usize, 
        window_ptr: usize, 
    }

    pub struct WindowSize {
        pub width: u32,
        pub height: u32,
    }
    
    pub struct Vertex {
        pub pos: [f32; 3],
        pub light_level: [f32; 3],
        pub texture_pos: [f32; 2],
        pub texture_id: u32,
        pub sector_id: u32,
        pub colormap_idx: u32,
    }

    pub struct UniformBufferObject {
        pub model: [f32; 16],
        pub view: [f32; 16],
        pub proj: [f32; 16],
    }

    pub struct TextureDescriptor {
        pub width: u32,
        pub height: u32,
        pub pixel_offset: usize,
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
        unsafe fn startFrame(self: Pin<&mut VulkanRenderer>, ubo_ptr: *const UniformBufferObject);
        fn endFrame(self: Pin<&mut VulkanRenderer>);
        fn drawLevel(self: Pin<&mut VulkanRenderer>);
        fn drawObjects(self: Pin<&mut VulkanRenderer>);
        unsafe fn updateLevelGeometry(self: Pin<&mut VulkanRenderer>, vertices: *const Vertex, vertex_count: usize, indices: *const u16, index_count: usize);
        unsafe fn updateObjectGeometry(self: Pin<&mut VulkanRenderer>, vertices: *const Vertex, vertex_count: usize, indices: *const u16, index_count: usize);
        unsafe fn uploadPalettes(self: Pin<&mut VulkanRenderer>, palettes_ptr: *const f32);
        unsafe fn uploadColormap(self: Pin<&mut VulkanRenderer>, colormap_ptr: *const u8);
        unsafe fn uploadTextureArray(self: Pin<&mut VulkanRenderer>, descriptors: *const TextureDescriptor, descriptor_count: usize, all_pixels: *const u8, all_pixels_count: usize);
        fn setPaletteIndex(self: Pin<&mut VulkanRenderer>, idx: u32);
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

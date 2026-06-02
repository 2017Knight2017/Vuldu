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

    extern "Rust" {
        unsafe fn get_winit_window_size(window_raw_ptr: usize) -> WindowSize;
    }

    unsafe extern "C++" {
        include!("vulkan_renderer.h");

        type VulkanRenderer;

        fn create_renderer() -> UniquePtr<VulkanRenderer>;

        fn initVulkan(
            self: Pin<&mut VulkanRenderer>, 
            handles: &WindowHandles,
            window_raw_ptr: usize
        );
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
#[cxx::bridge]
pub mod ffi {
    struct WindowHandles {
        display_ptr: usize, 
        window_ptr: usize, 
    }

    unsafe extern "C++" {
        include!("vulkan_renderer.h");

        type VulkanRenderer;

        fn create_renderer(width: u32, height: u32) -> UniquePtr<VulkanRenderer>;

        fn init_vulkan(
            self: Pin<&mut VulkanRenderer>, 
            handles: &WindowHandles,
        ) -> bool;
    }
}
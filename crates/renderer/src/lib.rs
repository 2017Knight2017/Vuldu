mod bridge; 

pub use bridge::ffi::{WindowHandles, WindowSize, Vertex, UniformBufferObject, TextureDescriptor, ObjectInstance};
use bridge::ffi::{VulkanRenderer, createRenderer};
use std::pin::Pin;
use cxx::UniquePtr;

pub struct SafeRenderer {
    renderer: UniquePtr<VulkanRenderer>,
}

impl SafeRenderer {
    pub fn new() -> Self {
        Self {
            renderer: createRenderer(),
        }
    }

    fn pin_mut(&mut self) -> Pin<&mut VulkanRenderer> {
        self.renderer.pin_mut()
    }

    pub fn init(&mut self, handles: &WindowHandles, window_raw_ptr: usize) {
        self.pin_mut().initVulkan(handles, window_raw_ptr);
    }

    pub fn shutdown(&mut self) {
        self.pin_mut().cleanup();
    }

    pub fn recreate_swapchain(&mut self) {
        self.pin_mut().recreateSwapChain();
    }

    pub fn upload_palettes(&mut self, palettes: &[f32]) {
        unsafe {
            self.pin_mut().uploadPalettes(palettes.as_ptr());
        }
    }

    pub fn upload_colormap(&mut self, colormap: &[u8]) {
        unsafe {
            self.pin_mut().uploadColormap(colormap.as_ptr());
        }
    }

    pub fn set_palette_index(&mut self, idx: u32) {
        self.pin_mut().setPaletteIndex(idx);
    }

    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.pin_mut().setResolution(width, height);
    }

    pub fn update_level_geometry(&mut self, vertices: &[Vertex], indices: &[u32]) {
        unsafe {
            self.pin_mut().updateLevelGeometry(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
            );
        }
    }

    pub fn update_object_geometry(&mut self, vertices: &[Vertex], indices: &[u32]) {
        unsafe {
            self.pin_mut().updateObjectGeometry(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
            );
        }
    }

    pub fn update_object_instances(&mut self, instances: &[ObjectInstance]) {
        unsafe {
            self.pin_mut().updateObjectInstances(instances.as_ptr(), instances.len());   
        }
    }

    pub fn start_frame(&mut self, ubo: &UniformBufferObject) {
        unsafe {
            self.pin_mut().startFrame(ubo as *const UniformBufferObject);
        }
    }

    pub fn end_frame(&mut self) {
        self.pin_mut().endFrame();
    }

    pub fn draw_level(&mut self) {
        self.pin_mut().drawLevel();
    }

    pub fn draw_objects(&mut self) {
        self.pin_mut().drawObjects();
    }

    pub fn upload_texture_array(&mut self, descriptors: &[TextureDescriptor], all_pixels: &[u8]) {
        unsafe {
            self.pin_mut().uploadTextureArray(descriptors.as_ptr(), descriptors.len(), all_pixels.as_ptr(), all_pixels.len());
        }
    }
}

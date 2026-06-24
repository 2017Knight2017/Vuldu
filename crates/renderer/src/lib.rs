mod bridge; 

pub use bridge::ffi::{WindowHandles, WindowSize, Vertex, UniformBufferObject, TextureDescriptor};
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

    pub fn update_level_geometry(&mut self, vertices: &[Vertex], indices: &[u16]) {
        unsafe {
            self.pin_mut().updateLevelGeometry(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
            );
        }
    }

    pub fn update_object_geometry(&mut self, vertices: &[Vertex], indices: &[u16]) {
        unsafe {
            self.pin_mut().updateObjectGeometry(
                vertices.as_ptr(),
                vertices.len(),
                indices.as_ptr(),
                indices.len(),
            );
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

    pub fn upload_texture_array(
        &mut self, 
        descriptors: &[TextureDescriptor], 
        descriptor_count: usize, 
        all_pixels: &[u8], 
        all_pixels_count: usize
    ) {
        unsafe {
            self.pin_mut().uploadTextureArray(descriptors.as_ptr(), descriptor_count, all_pixels.as_ptr(), all_pixels_count);
        }
    }
}

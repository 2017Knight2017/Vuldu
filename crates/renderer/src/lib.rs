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

    pub fn add_texture(&mut self, pixels: &[u8], width: u32, height: u32) -> u32 {
        unsafe {
            self.pin_mut().addTexture(pixels.as_ptr(), width, height)
        }
    }

    pub fn update_geometry(&mut self, vertices: &[Vertex], indices: &[u16]) {
        unsafe {
            self.pin_mut().updateGeometry(
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

    pub fn draw_sprite(
        &mut self, 
        texture_id: u32, width: u32, height: u32, 
        light_level: u32, left_offset: i16, top_offset: i16, 
        x: f32, y: f32, z: f32
    ) {
        self.pin_mut().drawSprite(
            texture_id, width, height, 
            light_level, left_offset, top_offset, 
            x, y, z
        );
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

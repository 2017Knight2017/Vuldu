mod bridge;

pub use bridge::ffi::{
	AnimLevelInfo, ObjectInstance, TextureDescriptor, UiInstance, UniformBufferObject, Vertex,
	WindowHandles, WindowSize,
};
use bridge::ffi::{VulkanRenderer, createRenderer, getAnimInfoSize, getMaxSky};
use cxx::UniquePtr;
use std::{pin::Pin, sync::OnceLock};

pub static MAX_SKY: OnceLock<usize> = OnceLock::new();
pub static ANIM_INFO_SIZE: OnceLock<usize> = OnceLock::new();

pub struct SafeRenderer {
	renderer: UniquePtr<VulkanRenderer>,
}

impl Default for SafeRenderer {
	fn default() -> Self {
		Self::new()
	}
}

impl SafeRenderer {
	pub fn new() -> Self {
		let _ = MAX_SKY.set(getMaxSky());
		let _ = ANIM_INFO_SIZE.set(getAnimInfoSize());
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

	pub fn upload_palettes(&mut self, palettes: &[u8]) {
		unsafe {
			self.pin_mut()
				.uploadPalettes(palettes.as_ptr(), palettes.len());
		}
	}

	pub fn upload_colormap(&mut self, colormap: &[u8]) {
		unsafe {
			self.pin_mut()
				.uploadColormap(colormap.as_ptr(), colormap.len());
		}
	}

	pub fn set_palette_index(&mut self, idx: u32) {
		self.pin_mut().setPaletteIndex(idx);
	}

	pub fn set_sky_index(&mut self, idx: u32) {
		self.pin_mut().setSkyIndex(idx);
	}

	pub fn set_global_timer(&mut self, global_timer: u32) {
		self.pin_mut().setGlobalTimer(global_timer);
	}

	pub fn set_camera_yaw(&mut self, camera_yaw: f32) {
		self.pin_mut().setCameraYaw(camera_yaw);
	}

	pub fn set_flags(&mut self, wireframe: bool, byte_shadows: bool) {
		self.pin_mut().setFlags(wireframe, byte_shadows);
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

	pub fn update_ui_geometry(&mut self, vertices: &[Vertex], indices: &[u32]) {
		unsafe {
			self.pin_mut().updateUiGeometry(
				vertices.as_ptr(),
				vertices.len(),
				indices.as_ptr(),
				indices.len(),
			);
		}
	}

	pub fn update_object_instances(&mut self, instances: &[ObjectInstance]) {
		unsafe {
			self.pin_mut()
				.updateObjectInstances(instances.as_ptr(), instances.len());
		}
	}

	pub fn update_ui_instances(&mut self, instances: &[UiInstance]) {
		unsafe {
			self.pin_mut()
				.updateUiInstances(instances.as_ptr(), instances.len());
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

	pub fn draw_ui(&mut self) {
		self.pin_mut().drawUi();
	}

	pub fn upload_texture_array(
		&mut self,
		descriptors: &[TextureDescriptor],
		all_pixels: &[u8],
		sky_widths: &[f32],
	) {
		unsafe {
			self.pin_mut().uploadTextureArray(
				descriptors.as_ptr(),
				descriptors.len(),
				all_pixels.as_ptr(),
				all_pixels.len(),
				sky_widths.as_ptr(),
				sky_widths.len(),
			);
		}
	}

	pub fn upload_anim_level_info(&mut self, info: &[AnimLevelInfo]) {
		unsafe {
			self.pin_mut()
				.uploadAnimLevelInfo(info.as_ptr(), info.len());
		}
	}
}

#[cxx::bridge]
pub(crate) mod ffi {
	pub struct WindowHandles {
		display_ptr: usize,
		window_ptr: usize,
		is_x11: bool,
	}

	pub struct WindowSize {
		pub width: u32,
		pub height: u32,
	}

	// SpriteVertex MUST equal to wad_parser::GpuSpriteVertex
	pub struct SpriteVertex {
		pub pos: [f32; 3],
		pub texture_pos: [f32; 2],
	}

	// LevelVertex MUST equal to wad_parser::GpuLevelVertex
	pub struct LevelVertex {
		pub pos: [f32; 3],
		pub texture_pos: [f32; 2],
		pub light_level: u32,
		pub texture_id: u32,
		pub floor_tex_id: u32,
		pub scroll_dir: f32,
		pub plane_a: u32,
		pub plane_b: u32,
		pub inv_tex_h: f32,
	}

	pub struct MVP {
		pub model: [f32; 16],
		pub view: [f32; 16],
		pub proj: [f32; 16],
	}

	pub struct ObjectInstance {
		pub pos: [f32; 3],
		pub sprite_offset: [f32; 2],
		pub sprite_size: [f32; 2],
		pub light_level: u32,
		pub texture_id: u32,
	}

	pub struct UiInstance {
		pub pos: [f32; 2],
		pub sprite_size: [f32; 2],
		pub texture_id: u32,
	}

	pub struct TextureDescriptor {
		pub width: u32,
		pub height: u32,
		pub pixel_offset: usize,
	}

	pub struct AnimLevelInfo {
		pub texture: u32,
		pub frames: u32,
		pub _padding: [u32; 2],
	}

	unsafe extern "C++" {
		include!("renderer.h");
		include!("utils.h");

		type VulkanRenderer;

		fn createRenderer() -> UniquePtr<VulkanRenderer>;
		fn initVulkan(
			self: Pin<&mut VulkanRenderer>,
			handles: &WindowHandles,
			width: u32,
			height: u32,
		);
		fn cleanup(self: Pin<&mut VulkanRenderer>);
		fn recreateSwapChain(self: Pin<&mut VulkanRenderer>, width: u32, height: u32);
		fn startFrame(self: Pin<&mut VulkanRenderer>, mvp: &MVP);
		fn endFrame(self: Pin<&mut VulkanRenderer>);
		fn drawLevel(self: Pin<&mut VulkanRenderer>);
		fn drawObjects(self: Pin<&mut VulkanRenderer>);
		fn drawUi(self: Pin<&mut VulkanRenderer>);
		fn updateLevelGeometry(
			self: Pin<&mut VulkanRenderer>,
			vertices: &[LevelVertex],
			indices: &[u32],
		);
		fn updateObjectGeometry(
			self: Pin<&mut VulkanRenderer>,
			vertices: &[SpriteVertex],
			indices: &[u32],
		);
		fn updateUiGeometry(
			self: Pin<&mut VulkanRenderer>,
			vertices: &[SpriteVertex],
			indices: &[u32],
		);
		fn updateObjectInstances(self: Pin<&mut VulkanRenderer>, instances: &[ObjectInstance]);
		fn updateUiInstances(self: Pin<&mut VulkanRenderer>, instances: &[UiInstance]);
		fn uploadPalettes(self: Pin<&mut VulkanRenderer>, palettes: &[u8]);
		fn uploadColormap(self: Pin<&mut VulkanRenderer>, colormap: &[u8]);
		fn uploadTextureArray(
			self: Pin<&mut VulkanRenderer>,
			descriptors: &[TextureDescriptor],
			pixels: &[u8],
			sky_widths: &[f32],
		);
		fn initSectorHeights(self: Pin<&mut VulkanRenderer>, heights: &[f32]);
		fn updateSectorHeights(self: Pin<&mut VulkanRenderer>, heights: &[f32]);
		fn uploadAnimLevelInfo(self: Pin<&mut VulkanRenderer>, info: &[AnimLevelInfo]);
		fn setPaletteIndex(self: Pin<&mut VulkanRenderer>, idx: u32);
		fn getPaletteIndex(self: Pin<&mut VulkanRenderer>) -> u32;
		fn setSkyIndex(self: Pin<&mut VulkanRenderer>, idx: u32);
		fn setFlags(self: Pin<&mut VulkanRenderer>, flags_to_invert: u32);
		fn setGlobalTimer(self: Pin<&mut VulkanRenderer>, global_timer: u32);
		fn setCameraYaw(self: Pin<&mut VulkanRenderer>, camera_yaw: f32);
		fn getMaxSky() -> usize;
		fn getAnimInfoSize() -> usize;
	}
}

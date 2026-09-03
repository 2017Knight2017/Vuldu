mod bridge;

pub use bridge::ffi::{
	AnimLevelInfo, LevelVertex, MVP, ObjectInstance, SpriteVertex, TextureDescriptor, UiInstance,
	WindowHandles, WindowSize,
};
use bridge::ffi::{VulkanRenderer, createRenderer, getAnimInfoSize, getMaxSky};
use cxx::UniquePtr;
use std::{pin::Pin, sync::LazyLock};

pub static MAX_SKY: LazyLock<usize> = LazyLock::new(getMaxSky);
pub static ANIM_INFO_SIZE: LazyLock<usize> = LazyLock::new(getAnimInfoSize);

pub struct SafeRenderer(UniquePtr<VulkanRenderer>);

impl Default for SafeRenderer {
	fn default() -> Self {
		Self::new()
	}
}

impl SafeRenderer {
	pub fn new() -> Self {
		Self(createRenderer())
	}

	pub fn pin(&mut self) -> Pin<&mut VulkanRenderer> {
		self.0.pin_mut()
	}
}

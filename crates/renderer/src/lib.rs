mod bridge;

pub use bridge::ffi::{
	AnimLevelInfo, LevelVertex, MVP, ObjectInstance, SpriteVertex, TextureDescriptor, UiInstance,
	WindowHandles, WindowSize,
};
use bridge::ffi::{VulkanRenderer, createRenderer, getAnimInfoSize, getMaxSky};
use cxx::UniquePtr;
use std::{pin::Pin, sync::OnceLock};

pub static MAX_SKY: OnceLock<usize> = OnceLock::new();
pub static ANIM_INFO_SIZE: OnceLock<usize> = OnceLock::new();

pub struct SafeRenderer(UniquePtr<VulkanRenderer>);

impl Default for SafeRenderer {
	fn default() -> Self {
		Self::new()
	}
}

impl SafeRenderer {
	pub fn new() -> Self {
		let _ = MAX_SKY.set(getMaxSky());
		let _ = ANIM_INFO_SIZE.set(getAnimInfoSize());
		Self(createRenderer())
	}

	pub fn pin(&mut self) -> Pin<&mut VulkanRenderer> {
		self.0.pin_mut()
	}
}

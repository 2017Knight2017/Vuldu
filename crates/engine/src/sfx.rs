#[derive(Debug, Clone, Copy)]
pub struct SfxEvent {
	pub sfx_id: u64,
	pub pos: Option<(f32, f32, f32)>,
}

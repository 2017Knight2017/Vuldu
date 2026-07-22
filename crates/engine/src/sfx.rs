#[derive(Debug, Clone, Copy)]
pub struct SfxEvent {
    pub sfx_id: u64,
    pub position: Option<(f32, f32)>,
}

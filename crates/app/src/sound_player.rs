use engine::{PlayerRotation, Position, SfxEvent, aprox_xyz_distance};
use rodio::Source;
use rustc_hash::FxHashMap;
use wad_parser::DoomSfx;
use rodio::{
    MixerDeviceSink, SpatialPlayer, buffer::SamplesBuffer, math::nz
};
use std::num::NonZero;
use std::f32::consts::TAU;
use hecs::World;

const EAR_HALF_WIDTH: f32 = 0.08;
const MAX_AUDIBLE_DIST: f32 = 600.0;
const METERS_PER_UNIT: f32 = 0.03;
const VOLUME_FAR: f32 = 22.0;
const VOLUME_NEAR: f32 = 3.0;
const METERS_DIST_CAP: f32 = 2.3;

pub struct DoomSfxPlayer {
    spatial_players: Vec<SpatialPlayer>,
    idx: usize
}

pub struct AudioContext {
    _audio_stream_handle: MixerDeviceSink,
    pub data: FxHashMap<u64, DoomSfx>,
    pub buffer: Vec<SfxEvent>,
    pub player: DoomSfxPlayer,
}

impl DoomSfxPlayer {
    pub fn new(handle: &MixerDeviceSink) -> Self {
        DoomSfxPlayer {
            spatial_players: Vec::from_iter([
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
                
                // the ninth one is reserved for sounds in player's head
                SpatialPlayer::connect_new(handle.mixer(), [0.0; 3], [-EAR_HALF_WIDTH, 0.0, 0.0], [EAR_HALF_WIDTH, 0.0, 0.0]),
            ]),
            idx: 0
        }
    }

    fn play(&mut self, src: SamplesBuffer, delta_pos: [f32; 3], left_ear: [f32; 3], right_ear: [f32; 3], volume_factor: f32) {
        for _ in 0..8 {
            if self.spatial_players[self.idx].empty() {
                self.spatial_players[self.idx].set_emitter_position(delta_pos);
                self.spatial_players[self.idx].set_left_ear_position(left_ear);
                self.spatial_players[self.idx].set_right_ear_position(right_ear);
                self.spatial_players[self.idx].append(src.amplify(volume_factor));
                
                self.idx = (self.idx + 1) & 0b111;
                return;
            }
            self.idx = (self.idx + 1) & 0b111;
        }
    }

    fn play_head_sound(&mut self, src: SamplesBuffer) {
        if !self.spatial_players[8].empty() {
            return;
        }

        self.spatial_players[8].append(src);
    }
}

impl AudioContext {
    pub fn new() -> Result<Self, String> {
        let mut _audio_stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
            .map_err(|_| "Failed to create an audio stream handle".to_string())?;
        _audio_stream_handle.log_on_drop(false);

        Ok(Self { 
            player: DoomSfxPlayer::new(&_audio_stream_handle), 
            _audio_stream_handle, 
            data: FxHashMap::default(), 
            buffer: Vec::new(), 
        })
    }

    pub fn system(&mut self, world: &World) {
        let mut query = world.query::<(&Position, &PlayerRotation)>();
        let (p_pos, p_rot) = match query.iter().next() {
            Some(player) => player,
            None => return,
        };

        for event in self.buffer.drain(..) {
            let sound = match self.data.get(&event.sfx_id) {
                Some(sound) => sound,
                None => continue
            };

            match event.pos {
                None => {
                    let source = SamplesBuffer::new(nz!(1), NonZero::new(sound.sample_rate).unwrap(), sound.samples.clone());
                    self.player.play_head_sound(source);
                },
                Some(emitter_pos) => {
                    if self.player.spatial_players[..8].iter().all(|p| !p.empty()) {
                        continue;
                    }

                    let approx_dist = aprox_xyz_distance((p_pos.x, p_pos.y, p_pos.z), emitter_pos);
                    if approx_dist > MAX_AUDIBLE_DIST {
                        continue;
                    }

                    let mut dx_m = (p_pos.x - emitter_pos.0) * METERS_PER_UNIT;
                    let mut dy_m = (p_pos.y - emitter_pos.1) * METERS_PER_UNIT;
                    let mut dz_m = (p_pos.z - emitter_pos.2) * METERS_PER_UNIT;

                    if dx_m.abs() < METERS_DIST_CAP { dx_m = METERS_DIST_CAP * dx_m.signum(); }
                    if dy_m.abs() < METERS_DIST_CAP { dy_m = METERS_DIST_CAP * dy_m.signum(); }
                    if dz_m.abs() < METERS_DIST_CAP { dz_m = METERS_DIST_CAP * dz_m.signum(); }

                    let source = SamplesBuffer::new(nz!(1), NonZero::new(sound.sample_rate).unwrap(), sound.samples.clone());

                    let p_angle = (p_rot.angle as f64 / u32::MAX as f64) as f32 * TAU;
                    let perp_x = p_angle.sin() * EAR_HALF_WIDTH;
                    let perp_z = -p_angle.cos() * EAR_HALF_WIDTH;

                    let volume_factor = VOLUME_NEAR + approx_dist / MAX_AUDIBLE_DIST * (VOLUME_FAR - VOLUME_NEAR);

                    self.player.play(
                        source, 
                        [dx_m, dy_m, dz_m], 
                        [perp_x, 0.0, perp_z],
                        [-perp_x, 0.0, -perp_z],
                        volume_factor
                    );
                }
            }
        }
    }
}

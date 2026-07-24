use engine::{Position, PlayerRotation, SfxEvent};
use rodio::Source;
use rustc_hash::FxHashMap;
use wad_parser::DoomSfx;
use rodio::{
    MixerDeviceSink, SpatialPlayer, buffer::SamplesBuffer, math::nz
};
use std::num::NonZero;
use std::f32::consts::TAU;
use hecs::QueryBorrow;

pub struct DoomSfxPlayer {
    spatial_players: Vec<SpatialPlayer>,
    idx: usize
}

const EAR_HALF_WIDTH: f32 = 0.08;
const MAX_AUDIBLE_DIST: f32 = 600.0;
const METERS_PER_UNIT: f32 = 0.03;

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

    fn play(&mut self, src: SamplesBuffer, delta_pos: [f32; 3], left_ear: [f32; 3], right_ear: [f32; 3]) {
        for _ in 0..8 {
            if self.spatial_players[self.idx].empty() {
                self.spatial_players[self.idx].set_emitter_position(delta_pos);
                self.spatial_players[self.idx].set_left_ear_position(left_ear);
                self.spatial_players[self.idx].set_right_ear_position(right_ear);
                self.spatial_players[self.idx].append(src.amplify(22.0));
                
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

pub fn audio_system(
    mut audio_query: QueryBorrow<'_, (&Position, &PlayerRotation)>,
    audio_buffer: &mut Vec<SfxEvent>,
    audio_player: &mut DoomSfxPlayer,
    sound_cache: &FxHashMap<u64, DoomSfx>,
) {
    let (p_pos, p_rot) = match audio_query.iter().next() {
        Some(player) => player,
        None => return,
    };

    for event in audio_buffer.drain(..) {
        let sound = match sound_cache.get(&event.sfx_id) {
            Some(win) => win,
            None => continue
        };

        match event.position {
            None => {
                let source = SamplesBuffer::new(nz!(1), NonZero::new(sound.sample_rate).unwrap(), sound.samples.clone());
                audio_player.play_head_sound(source);
            },
            Some(emitter_pos) => {
                if audio_player.spatial_players[..8].iter().all(|p| !p.empty()) {
                    continue;
                }

                let dx = (p_pos.x - emitter_pos.0).abs();
                let dy = (p_pos.y - emitter_pos.1).abs();
                let dz = (p_pos.z - emitter_pos.2).abs();

                let approx_dist_xz = dx + dz - (dx.min(dz) * 0.5);
                
                let approx_dist = approx_dist_xz + (dy * 0.5);

                if approx_dist > MAX_AUDIBLE_DIST {
                    continue;
                }

                let dx_m = (p_pos.x - emitter_pos.0) * METERS_PER_UNIT;
                let dy_m = (p_pos.y - emitter_pos.1) * METERS_PER_UNIT;
                let dz_m = (p_pos.z - emitter_pos.2) * METERS_PER_UNIT;

                let source = SamplesBuffer::new(nz!(1), NonZero::new(sound.sample_rate).unwrap(), sound.samples.clone());

                let p_angle = (p_rot.angle as f64 / u32::MAX as f64) as f32 * TAU;
                let perp_x = p_angle.sin() * EAR_HALF_WIDTH;
                let perp_z = -p_angle.cos() * EAR_HALF_WIDTH;

                audio_player.play(
                    source, 
                    [dx_m, dy_m, dz_m], 
                    [perp_x, 0.0, perp_z],
                    [-perp_x, 0.0, -perp_z]
                );
            }
        }
    }
}

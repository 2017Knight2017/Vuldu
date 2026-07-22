use engine::SfxEvent;
use rustc_hash::FxHashMap;
use wad_parser::DoomSfx;
use rodio::{
    buffer::SamplesBuffer, math::nz, mixer::Mixer, source::Source
};
use std::num::NonZero;

const CLOSE_DIST: f32 = 160.0;
const CLIPPING_DIST: f32 = 1200.0;
const MAX_CONCURRENT_SOUNDS_PER_TICK: usize = 8;

pub fn calculate_spatial_volume(listener_pos: (f32, f32), source_pos: (f32, f32)) -> f32 {
    let dx = (listener_pos.0 - source_pos.0).abs();
    let dy = (listener_pos.1 - source_pos.1).abs();
    
    let approx_dist = dx + dy - (dx.min(dy) * 0.5);

    if approx_dist > CLIPPING_DIST {
        return 0.0;
    }

    if approx_dist < CLOSE_DIST {
        return 1.0;
    }

    (CLIPPING_DIST - approx_dist) / (CLIPPING_DIST - CLOSE_DIST)
}

pub fn audio_system(
    audio_buffer: &mut Vec<SfxEvent>,
    sound_cache: &FxHashMap<u64, DoomSfx>,
    mixer: &Mixer,
    player_pos: (f32, f32),
) {
    let mut played_counts: FxHashMap<u64, usize> = FxHashMap::default();
    let mut total_played_this_tick = 0;

    for event in audio_buffer.drain(..) {
        if total_played_this_tick >= MAX_CONCURRENT_SOUNDS_PER_TICK {
            break;
        }

        let count = played_counts.entry(event.sfx_id).or_insert(0);
        if *count >= 2 { continue; }

        if let Some(sound) = sound_cache.get(&event.sfx_id) {
            let final_volume = match event.position {
                Some(emitter_pos) => calculate_spatial_volume(player_pos, emitter_pos),
                None => 1.0
            };

            if final_volume <= 0.1 { continue; }

            let source = SamplesBuffer::new(nz!(1), NonZero::new(sound.sample_rate).unwrap(), sound.samples.clone()).amplify(final_volume);

            mixer.add(source);
            *count += 1;
            total_played_this_tick += 1;
        }
    }
}

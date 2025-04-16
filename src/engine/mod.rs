use crate::tracker::Tracker;
use crate::Song;
use mod_engine::ModEngine;

mod mod_engine;

pub enum Engine {
    Mod(mod_engine::ModEngine),
}

pub trait TrackerEngine {
    fn next_tick(&mut self);
    fn is_finished(&self) -> bool;
    fn get_audio_buffer(&mut self, buffer: &mut [f32]);

    fn samples_since_tick(&self) -> usize;
    fn set_samples_since_tick(&mut self, value: usize);
    fn samples_per_tick(&self) -> usize;
    fn set_samples_per_tick(&mut self, value: usize);
}

impl TrackerEngine for Engine {
    fn next_tick(&mut self) {
        match self {
            Engine::Mod(e) => e.next_tick(),
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Engine::Mod(e) => e.is_finished(),
        }
    }

    fn get_audio_buffer(&mut self, buffer: &mut [f32]) {
        match self {
            Engine::Mod(e) => e.get_audio_buffer(buffer),
        }
    }

    fn samples_since_tick(&self) -> usize {
        match self {
            Engine::Mod(e) => e.samples_since_tick(),
        }
    }
    fn set_samples_since_tick(&mut self, value: usize) {
        match self {
            Engine::Mod(e) => e.set_samples_since_tick(value),
        }
    }
    fn samples_per_tick(&self) -> usize {
        match self {
            Engine::Mod(e) => e.samples_per_tick(),
        }
    }
    fn set_samples_per_tick(&mut self, value: usize) {
        match self {
            Engine::Mod(e) => e.set_samples_per_tick(value),
        }
    }
}

impl Engine {
    pub fn new(
        song: Song,
        sample_rate: cpal::SampleRate,
        channel_count: cpal::ChannelCount,
    ) -> Engine {
        match song.metadata.tracker {
            Tracker::ProTracker | Tracker::NoiseTracker => {
                Engine::Mod(ModEngine::new(song, sample_rate, channel_count))
            }

            _ => todo!(),
        }
    }
}

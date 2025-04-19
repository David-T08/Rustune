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

    fn sample_rate(&self) -> u32;
    fn set_sample_rate(&mut self, value: u32);

    fn channel_count(&self) -> u16;
    fn set_channel_count(&mut self, value: u16);

    fn tick_duration(&self) -> f32;
}

// could probably simplify a lot of this with a macro
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

    fn sample_rate(&self) -> u32 {
        match self {
            Engine::Mod(e) => e.sample_rate(),
        }
    }

    fn set_sample_rate(&mut self, value: u32) {
        match self {
            Engine::Mod(e) => e.set_sample_rate(value),
        }
    }

    fn channel_count(&self) -> u16 {
        match self {
            Engine::Mod(e) => e.channel_count(),
        }
    }

    fn set_channel_count(&mut self, value: u16) {
        match self {
            Engine::Mod(e) => e.set_channel_count(value),
        }
    }

    fn tick_duration(&self) -> f32 {
        match self {
            Engine::Mod(e) => e.tick_duration(),
        }
    }
}

impl Engine {
    pub fn new(song: Song) -> Engine {
        match song.metadata.tracker {
            Tracker::ProTracker | Tracker::NoiseTracker => Engine::Mod(ModEngine::new(song)),

            _ => todo!(),
        }
    }
}

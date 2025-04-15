use std::time::Duration;

use crate::tracker::Tracker;
use crate::Song;
use mod_engine::ModEngine;

mod mod_engine;

pub enum Engine<'a> {
    Mod(mod_engine::ModEngine<'a>),
}

pub trait TrackerEngine {
    fn next_tick(&mut self);
    fn sleep_duration(&self) -> Duration;
    fn is_finished(&self) -> bool;
    fn get_audio_buffer(&mut self, buffer: &mut [f32]);
}

impl TrackerEngine for Engine<'_> {
    fn next_tick(&mut self) {
        match self {
            Engine::Mod(e) => e.next_tick(),
        }
    }

    fn sleep_duration(&self) -> Duration {
        match self {
            Engine::Mod(e) => e.sleep_duration(),
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
}

impl Engine<'_> {
    pub fn new(song: &Song) -> Engine<'_> {
        match song.metadata.tracker {
            Tracker::ProTracker | Tracker::NoiseTracker => Engine::Mod(ModEngine::new(song)),

            _ => todo!(),
        }
    }
}

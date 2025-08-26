use crate::tracker::Tracker;
use crate::Song;

mod mod_engine;

use mod_engine::ModEngine;

pub enum Engine {
    Mod(ModEngine),
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

macro_rules! impl_tracker {
    ($enum:ident :: $variant:ident => $e:ident $( : $inner_ty:ty )? ) => {
        impl TrackerEngine for $enum {
            fn next_tick(&mut self) {
                let $enum::$variant($e) = self;
                $e.next_tick();

                // let sst = $e.samples_since_tick();
                // let spt = $e.samples_per_tick();
                // $e.set_samples_since_tick(sst % spt);
            }

            fn is_finished(&self) -> bool {
                let $enum::$variant($e) = self;
                $e.is_finished()
            }

            fn get_audio_buffer(&mut self, buffer: &mut [f32]) {
                let $enum::$variant($e) = self;
                $e.get_audio_buffer(buffer)
            }

            fn samples_since_tick(&self) -> usize {
                let $enum::$variant($e) = self;
                $e.samples_since_tick()
            }

            fn set_samples_since_tick(&mut self, value: usize) {
                let $enum::$variant($e) = self;
                $e.set_samples_since_tick(value)
            }

            fn samples_per_tick(&self) -> usize {
                let $enum::$variant($e) = self;
                $e.samples_per_tick()
            }

            fn sample_rate(&self) -> u32 {
                let $enum::$variant($e) = self;
                $e.sample_rate()
            }

            fn set_sample_rate(&mut self, value: u32) {
                let $enum::$variant($e) = self;
                $e.set_sample_rate(value)
            }

            fn channel_count(&self) -> u16 {
                let $enum::$variant($e) = self;
                $e.channel_count()
            }

            fn set_channel_count(&mut self, value: u16) {
                let $enum::$variant($e) = self;
                $e.set_channel_count(value)
            }

            fn tick_duration(&self) -> f32 {
                let $enum::$variant($e) = self;
                $e.tick_duration()
            }
        }
    }
}

impl_tracker!(Engine::Mod => e);

impl Engine {
    pub fn new(song: Song) -> Engine {
        match song.metadata.tracker {
            Tracker::ProTracker | Tracker::NoiseTracker => Engine::Mod(ModEngine::new(song)),

            _ => todo!(),
        }
    }
}

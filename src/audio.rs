use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use std::thread;

use crate::engine::{Engine, TrackerEngine};

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Auto,
    NoAudio,
}

pub struct Playback {
    done_rx: Receiver<()>,

    inner: Inner,
}

enum Inner {
    Silent { handle: thread::JoinHandle<()> },

    #[cfg(feature = "audio")]
    Cpal { _stream: cpal::Stream },
}

impl Playback {
    pub fn wait(self) {
        let _ = self.done_rx.recv();
    }
}

pub fn start(mut engine: Engine, mode: Mode) -> Result<Playback, Box<dyn std::error::Error>> {
    match mode {
        Mode::NoAudio => Ok(start_silent(engine)),
        Mode::Auto => {
            #[cfg(feature = "audio")]
            {
                match try_start_cpal(engine) {
                    Ok(pb) => return Ok(pb),
                    Err(e) => {
                        eprintln!("Audio not available ({e}). Falling back to silent mode.");
                    }
                }
            }

            Ok(start_silent(engine))
        }
    }
}

fn start_silent(mut engine: Engine) -> Playback {
    println!("No audio detected");
    let (tx_done, rx_done) = channel::<()>();

    let handle = thread::spawn(move || {
        loop {
            if engine.is_finished() {
                let _ = tx_done.send(());
                break;
            }

            engine.next_tick();
            thread::sleep(Duration::from_secs_f32(engine.tick_duration()));
        }
    });

    Playback {
        done_rx: rx_done,
        inner: Inner::Silent { handle },
    }
}

#[cfg(feature = "audio")]
fn try_start_cpal(mut engine: Engine) -> Result<Playback, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;

    let config = device
        .default_output_config()
        .map(cpal::StreamConfig::from)?;

    println!("Audio detected");

    engine.set_channel_count(config.channels);
    engine.set_sample_rate(config.sample_rate.0);

    let (tx_done, rx_done) = channel::<()>();
    let killswitch = tx_done.clone();

    let mut engine_in_cb = engine;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            engine_in_cb.get_audio_buffer(data);

            let frames_rendered = data.len() / config.channels as usize;

            let samples_since_tick = engine_in_cb.samples_since_tick();
            engine_in_cb.set_samples_since_tick(samples_since_tick + frames_rendered);

            while engine_in_cb.samples_since_tick() >= engine_in_cb.samples_per_tick() {
                engine_in_cb.next_tick();

                let sst = engine_in_cb.samples_since_tick();
                let spt = engine_in_cb.samples_per_tick();
                engine_in_cb.set_samples_since_tick(sst - spt);
            }

            if engine_in_cb.is_finished() {
                let _ = killswitch.send(());
            }
        },
        move |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    )?;

    stream.play()?;

    Ok(Playback {
        done_rx: rx_done,
        inner: Inner::Cpal { _stream: stream },
    })
}

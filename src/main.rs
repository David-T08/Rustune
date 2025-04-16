use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use clap::Parser;
use engine::{Engine, TrackerEngine};
use song::Song;

mod bytereader;
mod formats;
mod song;
mod tracker;

mod engine;

/// CLI Based tracker player
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("No output device available");

    let config: cpal::StreamConfig = device
        .default_output_config()
        .expect("Failed to get audio configuration")
        .into();

    let args = Args::parse();
    let track = Song::new(&args.path)?;

    let engine = Arc::new(Mutex::new(Engine::new(
        track,
        config.sample_rate,
        config.channels,
    )));

    let audio_engine = Arc::clone(&engine);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut engine = audio_engine.lock().unwrap();
            engine.get_audio_buffer(data);

            // Calculate how many frames (samples per channel) were rendered
            let frames_rendered = data.len() / config.channels as usize;

            let samples_since_tick = engine.samples_since_tick();
            engine.set_samples_since_tick(samples_since_tick + frames_rendered);

            // Advance tracker state as needed
            while engine.samples_since_tick() >= engine.samples_per_tick() {
                engine.next_tick();

                let samples_since_tick = engine.samples_since_tick();
                let samples_per_tick = engine.samples_per_tick();

                engine.set_samples_since_tick(samples_since_tick - samples_per_tick);
            }
        },
        move |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    )?;

    println!("Playing pattern: 0");
    stream.play()?;

    loop {
        if engine.lock().unwrap().is_finished() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}

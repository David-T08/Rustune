use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use clap::Parser;
use engine::{Engine, TrackerEngine};
use song::Song;

mod bytereader;
mod formats;
mod song;
mod tracker;

mod audio;
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

    // List all available output devices
    for device in host.output_devices()? {
        println!("Output device: {}", device.name()?);
    }

    let device = host
        .default_output_device()
        .expect("No output device available");

    let config = device
        .default_output_config()
        .expect("Failed to get audio configuration")
        .into();

    let args = Args::parse();
    let track = Song::new(&args.path)?;

    let engine = Arc::new(Mutex::new(Engine::new(track)));
    let audio_engine = Arc::clone(&engine);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut engine = audio_engine.lock().unwrap();
            engine.get_audio_buffer(data);
        },
        move |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    )?;

    stream.play()?;

    while let Ok(mut engine) = engine.lock() {
        if engine.is_finished() {
            break;
        }
        engine.next_tick();
        thread::sleep(engine.sleep_duration());
    }

    Ok(())
}

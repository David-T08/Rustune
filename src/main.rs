use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

    let config = device.default_output_config();

    let args = Args::parse();
    let track = Song::new(&args.path)?;

    let engine = Arc::new(Mutex::new(Engine::new(track)));

    if config.is_ok() {
        println!("Audio detected");
        println!("Playing pattern: 0");
        let config: cpal::StreamConfig = config.unwrap().into();

        let audio_engine = Arc::clone(&engine);
        engine.lock().unwrap().set_channel_count(config.channels);
        engine.lock().unwrap().set_sample_rate(config.sample_rate.0);

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

        stream.play()?;
        // Keep stream alive
        loop {
            if engine.lock().unwrap().is_finished() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    } else {
        println!("No audio detected");
        println!("Playing pattern: 0");

        let engine = Arc::clone(&engine);
        thread::spawn(move || loop {
            if engine.lock().unwrap().is_finished() {
                break;
            }

            engine.lock().unwrap().next_tick();

            std::thread::sleep(Duration::from_secs_f32(
                engine.lock().unwrap().tick_duration(),
            ));
        });
    }

    // Keep main thread alive
    loop {
        if engine.lock().unwrap().is_finished() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}

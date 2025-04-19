use std::path::PathBuf;
use std::sync::mpsc::channel;
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
    // I put this at the top so that we fail early on user input error
    let args = Args::parse();

    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or("No output device available")?;

    let track = Song::new(&args.path)?;

    let mut engine = Engine::new(track);

    // i would put most or all of the code below in a separate function, but thats a style choice imho

    // channel is used as a simple concurrency primitive: basic lock and key
    // common usage pattern for channels
    let (killswitch, blocker) = channel();

    if let Ok(config) = device.default_output_config().map(cpal::StreamConfig::from) {
        println!("Audio detected");
        println!("Playing pattern: 0");

        // engine is mutually exclusively used between either branch, so no
        // need to put it in an arc + mutex; we simply give ownership of it
        // to the branch that uses it
        engine.set_channel_count(config.channels);
        engine.set_sample_rate(config.sample_rate.0);

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                engine.get_audio_buffer(data);

                // Calculate how many frames (samples per channel) were rendered
                let frames_rendered = data.len() / config.channels as usize;

                let samples_since_tick = engine.samples_since_tick();
                engine.set_samples_since_tick(samples_since_tick + frames_rendered);

                // Advance tracker state as needed
                while engine.samples_since_tick() >= engine.samples_per_tick() {
                    engine.next_tick();

                    // i would personally move all this below logic into Engine::next_tick,
                    // since it depends on no outside info, and is always ran after next_tick
                    // but whether or not you do is up to you
                    let samples_since_tick = engine.samples_since_tick();
                    let samples_per_tick = engine.samples_per_tick();

                    engine.set_samples_since_tick(samples_since_tick - samples_per_tick);
                }

                // If playback is finished, kill the thread
                if engine.is_finished() {
                    killswitch.send(()).unwrap();
                }
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
                // should the program kill the main thread if an error is encountered?
                // killswitch.send(()).unwrap()
            },
            None,
        )?;

        stream.play()?;
    } else {
        println!("No audio detected");
        println!("Playing pattern: 0");

        thread::spawn(move || loop {
            if engine.is_finished() {
                break;
            }

            engine.next_tick();

            std::thread::sleep(Duration::from_secs_f32(engine.tick_duration()));
        });
    }

    // Keep stream alive; blocks until a message is received
    blocker.recv().unwrap();

    Ok(())
}

use std::path::PathBuf;
use std::thread;

use clap::Parser;
use song::Song;
use engine::{Engine, TrackerEngine};

mod bytereader;
mod formats;
mod song;
mod tracker;

mod engine;
mod audio;

/// CLI Based tracker player
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    path: PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let track = Song::new(&args.path)?;
    let mut engine = Engine::new(&track);

    while !engine.is_finished() {
        engine.next_tick();
        thread::sleep(engine.sleep_duration());
    }

    Ok(())
}

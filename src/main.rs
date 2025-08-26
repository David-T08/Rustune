use std::path::PathBuf;
use clap::Parser;

use engine::Engine;
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
    let args = Args::parse();
    let track = Song::new(&args.path)?;
    let engine = Engine::new(track);

    let pb = audio::start(engine, audio::Mode::Auto)?;
    println!("Playing pattern: 0");
    
    pb.wait();

    Ok(())
}

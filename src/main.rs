use std::path::PathBuf;

use clap::Parser;
use song::Song;

mod bytereader;
mod formats;
mod song;
mod tracker;

/// CLI Based tracker player
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    path: PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let _track = Song::new(&args.path)?;

    println!("Loaded!");

    Ok(())
}

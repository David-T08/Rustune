use std::path::PathBuf;

use clap::Parser;
use song::Song;

mod bytereader;
mod formats;
mod song;

/// CLI Based tracker player
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    #[clap(short, long)]
    path: PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let track = Song::new(&args.path)?;

    println!("Loaded!");

    Ok(())
}
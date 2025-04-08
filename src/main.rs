use clap::Parser;

mod bytereader;
mod formats;
mod song;

/// CLI Based tracker player
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to read
    path: String,
}

fn main() {
    let args = Args::parse();

    let track = song::new(&args.path);
    if let Err(e) = track {
      println!("{}", e);
      return
    }

    println!("Loaded!")
}
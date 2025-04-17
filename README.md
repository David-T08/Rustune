# Rustune
Rustune is simple, lightweight tracker in Rust. It's fully terminal based, meaning you can use it anywhere you'd like. It simply plays a file and shows all the notes being played, with audio output (if available).

### Supported Formats
- MOD

### Planned Support
- XM
- S3M
- IT

## Features
- Play songs and display their notes in real-time
- Simple terminal interface (**soon!**)
- Lightweight and fast
- Support for multiple formats

## Installation
To install Rustune, you need to have Rust and Cargo installed on your system. You can install Rust and Cargo by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install).
Once you have Rust and Cargo installed, you can clone the repository and build the project:
```bash
git clone 
```
```bash
cd rustune
cargo build --release
```
This will create an executable file in the `target/release` directory. You can run the player by executing the following command:
```bash
./target/release/rustune path/to/your/file.mod
```
## Todo
- Add Terminal UI
- Documentation
- Publish parts of code as a crate?
- Add support for XM, S3M, IT
- Error handling
- Refactor code once finished
- Add tests

## Disclaimer
This project is still in its early stages and may not work perfectly. If you encounter any issues, please open an issue on the GitHub repository. I will do my best to fix them as soon as possible.

I am quite new to rust and this is my first project. I would love to get feedback on my code and suggestions for improvements. If you have any ideas or suggestions, please feel free to open an issue or submit a pull request.
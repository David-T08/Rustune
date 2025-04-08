use crate::formats::modfile;
use std::{fs, fmt};

#[derive(Debug)]
pub enum SongError {
  IoError(String),
  ReadError(String),

}

impl fmt::Display for SongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SongError::IoError(msg) => write!(f, "IO Error: {}", msg),
            SongError::ReadError(msg) => write!(f, "Read Error: {}", msg),
        }
    }
}

impl SongError {
    pub fn read<S: Into<String>>(msg: S) -> Self {
        Self::ReadError(msg.into())
    }

    pub fn io<S: Into<String>>(msg: S) -> Self {
        Self::IoError(msg.into())
    }
}

impl std::error::Error for SongError {}

#[allow(dead_code)]
pub struct Song {
    pub name: String,

    pub sample_info: Vec<Sample>,
    pub pattern_table: Vec<i8>,
    pub num_patterns: i8,

    pub end_jump_pos: i8,
    pub tag: String,
}

#[allow(dead_code)]
pub struct Sample {
    pub name: String,
    pub length: i32,

    pub finetune: i8,
    pub volume: i8,

    pub repeat_offset: i32,
    pub repeat_length: i32,
}

pub fn new<'a>(path: &'a str) -> Result<Song, SongError> {
    println!("{}", path);
    
    // TODO: Handle multiple formats
    if path.split(".").last().unwrap_or("") != "mod" {
      return Err(SongError::io("Unrecognized format"));
    }

    let data = fs::read(path)
        .map_err(|_| SongError::io("Unrecognized format"))?;

    modfile::parse(data)
}

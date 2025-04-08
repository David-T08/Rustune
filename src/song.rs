use thiserror::Error;

use crate::formats::modfile;
use std::{ffi::OsStr, fs, path::Path};

#[derive(Debug, Error)]
pub enum SongError {
    #[error("IO Error: {0}")]
    Io(String),
    #[error("Read Error: {0}")]
    Read(String),
}

impl From<SongError> for String {
    fn from(value: SongError) -> Self {
        format!("{value}")
    }
}

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

impl Song {
    pub fn new(path: &Path) -> Result<Song, SongError> {
        dbg!(path);

        // TODO: Handle multiple formats
        if path.extension() != Some(OsStr::new("mod")) {
            return Err(SongError::Io("Unrecognized format".into()));
        }

        let data = fs::read(path).map_err(|_| SongError::Io("Unrecognized format".into()))?;

        modfile::song_from_bytes(data)
    }
}

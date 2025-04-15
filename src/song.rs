use thiserror::Error;

use crate::formats::modfile;
use crate::tracker::Tracker;
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

#[derive(Debug)]
#[allow(dead_code)]
pub enum PCMData {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SongMetadata {
    pub name: String,

    pub pattern_count: u8,
    pub channel_count: u8,

    pub samples: Vec<Sample>,
    pub pattern_table: Vec<u8>,

    pub format: String,
    pub end_jump: i8,

    pub tracker: Tracker,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Song {
    pub metadata: SongMetadata,

    pub patterns: Vec<Pattern>,
    pub samples: Vec<PCMData>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Sample {
    pub name: String,
    pub length: u16,

    pub finetune: i8,
    pub volume: u8,

    pub repeat_offset: u16,
    pub repeat_length: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Note {
    pub sample: u8,
    pub period: u16,

    pub effect: u8,
    pub argument: u8,
}

pub type Line = Vec<Note>;
pub type Pattern = Vec<Line>;

impl Song {
    pub fn new(path: &Path) -> Result<Song, SongError> {
        // TODO: Handle multiple formats
        if path.extension() != Some(OsStr::new("mod")) {
            return Err(SongError::Io("Unrecognized format".into()));
        }

        let data = fs::read(path).map_err(|_| SongError::Io("Unrecognized format".into()))?;
        let result = modfile::parse(data);

        result
    }
}

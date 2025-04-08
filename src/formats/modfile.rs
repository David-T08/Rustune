use crate::bytereader::{ByteReader, Encoding};
use crate::song::{self, Sample, Song, SongError};

pub fn read_sample(reader: &mut ByteReader) -> Result<Sample, song::SongError> {
    let _name = reader.read_str(22)?;
    let _length = reader.read_u16()? * 2;
    let _finetune = reader.read_bytes(1)?[0];

    todo!();
}

pub fn song_from_bytes(data: Vec<u8>) -> Result<Song, SongError> {
    let mut reader = ByteReader::new(&data, Encoding::BigEndian);

    let title = reader.read_str(20)?;

    let _sample1 = read_sample(&mut reader)?;

    dbg!(&title);

    todo!();
}

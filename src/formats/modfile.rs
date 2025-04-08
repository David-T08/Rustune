use crate::song;
use crate::bytereader::{ByteReader, Encoding};

fn read_sample(reader: &mut ByteReader) -> Result<song::Sample, song::SongError> {
  let _name = reader.read_str(22)?;
  let _length = reader.read_u16()? * 2;
  let _finetune = reader.read_bytes(1)?[0];
  
  todo!();
}

pub fn parse(data: Vec<u8>) -> Result<song::Song, song::SongError> {
  let mut reader = ByteReader::new(&data, Encoding::BigEndian);

  let title = reader.read_str(20)?;

  let _sample1 = read_sample(&mut reader)?;

  println!("Name: {title}");

  todo!();
}
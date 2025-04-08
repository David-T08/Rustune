use crate::song::SongError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Encoding {
    LittleEndian,
    BigEndian,
}

// look at `byteorder`, `bytes` crates for code that provides similar existing implementations

#[derive(Debug)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    offset: usize,
    encoding: Encoding,
}

trait ToArray {
    type Item;

    fn to_array<const N: usize>(self) -> [Self::Item; N];
}

impl ToArray for &[u8] {
    type Item = u8;

    /// Creates an array of the first N elements of the slice, automatically inferring
    /// the size from context
    fn to_array<const N: usize>(self) -> [Self::Item; N] {
        self[0..N].try_into().unwrap()
    }
}

#[allow(dead_code)]
impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8], encoding: Encoding) -> Self {
        ByteReader {
            data,
            encoding,
            offset: 0,
        }
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], SongError> {
        if self.offset + count > self.data.len() {
            return Err(SongError::Read(format!(
                "Not enough data to read {} bytes at offset {}",
                count, self.offset
            )));
        }

        let slice = &self.data[self.offset..self.offset + count];
        self.offset += count;

        Ok(slice)
    }

    pub fn read_str(&mut self, length: usize) -> Result<String, SongError> {
        let bytes = self.read_bytes(length).map_err(|_| {
            SongError::Read(format!(
                "Not enough data to read string of length {} at offset {}",
                length, self.offset
            ))
        })?;

        let string = std::str::from_utf8(bytes)
            .map_err(|e| SongError::Read(format!("UTF-8 error: {}", e)))?
            .to_string();

        Ok(string)
    }

    pub fn read_u8(&mut self) -> Result<u8, SongError> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, SongError> {
        Ok(self.read_bytes(1)?[0] as i8)
    }

    pub fn read_u16(&mut self) -> Result<u16, SongError> {
        let bytes = self.read_bytes(2)?;

        match self.encoding {
            Encoding::BigEndian => Ok(u16::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(u16::from_le_bytes(bytes.to_array())),
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, SongError> {
        let bytes = self.read_bytes(4)?;

        match self.encoding {
            Encoding::BigEndian => Ok(u32::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(u32::from_le_bytes(bytes.to_array())),
        }
    }

    pub fn read_i16(&mut self) -> Result<i16, SongError> {
        let bytes = self.read_bytes(2)?;

        match self.encoding {
            Encoding::BigEndian => Ok(i16::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(i16::from_le_bytes(bytes.to_array())),
        }
    }

    pub fn read_i32(&mut self) -> Result<i32, SongError> {
        let bytes = self.read_bytes(4)?;

        match self.encoding {
            Encoding::BigEndian => Ok(i32::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(i32::from_le_bytes(bytes.to_array())),
        }
    }
}

use crate::song::SongError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Encoding {
    LittleEndian,
    BigEndian,
}

#[derive(Debug)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    position: usize,
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
    /// Initializes a new `ByteReader` with the given byte slice and encoding.
    ///
    /// # Arguments
    /// * `data` - A slice of bytes to be read. See [`u8`] for more information
    /// * [`Encoding`] - The byte order to use when interpreting multi-byte values (e.g., `LittleEndian` or `BigEndian`).
    ///
    /// # Returns
    /// A new instance of `ByteReader` initialized with the provided data and encoding.
    pub fn new(data: &'a [u8], encoding: Encoding) -> Self {
        ByteReader {
            data,
            encoding,
            position: 0,
        }
    }

    /// Returns the byte order the reader was instantiated with
    ///
    /// # Returns
    /// A reference to the `Encoding` used by this `ByteReader`.
    /// * [`Encoding`] - The byte order
    pub fn encoding(&self) -> &Encoding {
        &self.encoding
    }

    /// Returns the current position the reader is at
    pub fn position(&self) -> usize {
        self.position
    }

    /// Seeks to a specific position in the byte stream.
    ///
    /// # Arguments
    /// * `position` - The new position (in bytes) to seek to.
    ///
    /// # Errors
    /// When the `position` is out of bounds.
    ///
    /// # Example
    /// ```
    /// let mut reader = ByteReader::new(&[0x01, 0x02, 0x03], Encoding::LittleEndian);
    /// assert_eq!(reader.seek(2).unwrap(), 0); // Moves to position 2, returns old position
    /// assert_eq!(reader.read_u8().unwrap, 0x03); // Reads the value at offset 2, which is 0x03
    /// ```
    pub fn seek(&mut self, position: usize) -> Result<usize, SongError> {
        let size = self.data.len();
        if position > size {
            return Err(SongError::Read(format!(
                "Out of bounds seek at {}; EOF: {}",
                position, size
            )));
        }

        let old_pos = self.position;
        self.position = position;

        Ok(old_pos)
    }

    /// Reads a chunk of bytes from the stream
    ///
    /// # Arguments
    /// * `count` - The amount of bytes to read
    ///
    /// # Errors
    /// When the reader tries to read more bytes than stored
    ///
    /// # Example
    /// ```rust
    /// let data: [u8; 12] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01];
    /// let mut reader = ByteReader::new(&data, Encoding::LittleEndian);
    ///
    /// assert_eq!(reader.read_bytes(6).unwrap(), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    /// assert_eq!(reader.read_bytes(6).unwrap(), [0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    /// ```
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], SongError> {
        if self.position + count > self.data.len() {
            return Err(SongError::Read(format!(
                "Not enough data to read {} bytes at offset {}",
                count, self.position
            )));
        }

        let slice = &self.data[self.position..self.position + count];
        self.position += count;

        Ok(slice)
    }

    /// Reads a string with the given length, trimming any null characters.
    ///
    /// # Arguments
    /// * `length` - The size of the string to read
    ///
    /// # Errors
    /// When the reader reads out of bounds.
    ///
    /// # Example
    /// ```
    /// let data: &[u8] = b"Hello, world!";
    /// let mut reader = ByteReader::new(&data, Encoding::LittleEndian);
    ///
    /// assert_eq!(reader.read_str(5).unwrap(), "Hello"); // Read "Hello"
    /// assert_eq!(reader.read_str(8).unwrap(), ", world!"); // Read ", world!"
    /// ```
    pub fn read_str(&mut self, length: usize) -> Result<String, SongError> {
        let bytes = self.read_bytes(length).map_err(|_| {
            SongError::Read(format!(
                "Not enough data to read string of length {} at offset {}",
                length, self.position
            ))
        })?;

        let string = std::str::from_utf8(bytes)
            .map_err(|e| SongError::Read(format!("UTF-8 error: {}", e)))?
            .trim_end_matches("\0")
            .to_string();

        Ok(string)
    }

    /// Read an unsigned byte
    ///
    /// # Errors
    /// When there is not enough data to be read
    pub fn read_u8(&mut self) -> Result<u8, SongError> {
        Ok(self.read_bytes(1)?[0])
    }

    /// Read a signed byte
    ///
    /// # Errors
    /// When there is not enough data to be read
    pub fn read_i8(&mut self) -> Result<i8, SongError> {
        Ok(self.read_bytes(1)?[0] as i8)
    }

    /// Read a unsigned 16-bit integer, accounting for the byteorder automatically
    ///
    /// # Errors
    /// When there is not enough data to be read
    pub fn read_u16(&mut self) -> Result<u16, SongError> {
        let bytes = self.read_bytes(2)?;

        match self.encoding {
            Encoding::BigEndian => Ok(u16::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(u16::from_le_bytes(bytes.to_array())),
        }
    }

    /// Read a signed 16-bit integer, accounting for the byteorder automatically
    ///
    /// # Errors
    /// When there is not enough data to be read
    pub fn read_i16(&mut self) -> Result<i16, SongError> {
        let bytes = self.read_bytes(2)?;

        match self.encoding {
            Encoding::BigEndian => Ok(i16::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(i16::from_le_bytes(bytes.to_array())),
        }
    }

    /// Read a unsigned 32-bit integer, accounting for the byteorder automatically
    ///
    /// # Errors
    /// When there is not enough data to be read
    pub fn read_u32(&mut self) -> Result<u32, SongError> {
        let bytes = self.read_bytes(4)?;

        match self.encoding {
            Encoding::BigEndian => Ok(u32::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(u32::from_le_bytes(bytes.to_array())),
        }
    }

    /// Read a signed 32-bit integer, accounting for the byteorder automatically
    ///
    /// # Errors
    /// When there is not enough data to be read
    pub fn read_i32(&mut self) -> Result<i32, SongError> {
        let bytes = self.read_bytes(4)?;

        match self.encoding {
            Encoding::BigEndian => Ok(i32::from_be_bytes(bytes.to_array())),
            Encoding::LittleEndian => Ok(i32::from_le_bytes(bytes.to_array())),
        }
    }
}

#[test]
fn test_seeking() {
    let mut reader = ByteReader::new(&[0x01, 0x02, 0x03], Encoding::LittleEndian);
    assert_eq!(reader.seek(2).unwrap(), 0); // Go to offset 2
    assert_eq!(reader.read_u8().unwrap(), 0x03); // Read byte
    assert_eq!(reader.seek(0).unwrap(), 3); // Go to the start of the file, seek returns last position
    assert_eq!(reader.read_u8().unwrap(), 0x01); // Read byte
    assert_eq!(reader.position(), 1); // Check we've correctly seeked
}

#[test]
fn test_sample_data() {
    // Define 16 bytes of sample data
    let data: [u8; 16] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ];

    // Test LittleEndian reader
    let mut le_reader = ByteReader::new(&data, Encoding::LittleEndian);
    assert_eq!(le_reader.read_u8().unwrap(), 0x01); // Read first byte
    assert_eq!(le_reader.read_u16().unwrap(), 0x0302); // Read next two bytes as LittleEndian u16
    assert_eq!(le_reader.read_u32().unwrap(), 0x07060504); // Read next four bytes as LittleEndian u32

    // Test BigEndian reader
    let mut be_reader = ByteReader::new(&data, Encoding::BigEndian);
    assert_eq!(be_reader.read_u8().unwrap(), 0x01); // Read first byte
    assert_eq!(be_reader.read_u16().unwrap(), 0x0203); // Read next two bytes as BigEndian u16
    assert_eq!(be_reader.read_u32().unwrap(), 0x04050607); // Read next four bytes as BigEndian u32
}

#[test]
fn read_multiple_bytes() {
    let data: [u8; 12] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
    ];

    let mut reader = ByteReader::new(&data, Encoding::LittleEndian);
    assert_eq!(
        reader.read_bytes(6).unwrap(),
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
    );
    assert_eq!(
        reader.read_bytes(6).unwrap(),
        [0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
    );
}

#[test]
fn read_string() {
    let data: &[u8] = b"Hello, world!";
    let mut reader = ByteReader::new(&data, Encoding::LittleEndian);

    assert_eq!(reader.read_str(5).unwrap(), "Hello"); // Read "Hello"
    assert_eq!(reader.read_str(8).unwrap(), ", world!"); // Read ", world!"
}

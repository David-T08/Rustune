use crate::bytereader::{ByteReader, Encoding};
use crate::song::{self, Sample, Song, SongError};
use crate::tracker::{self, Tracker};

fn read_sample(reader: &mut ByteReader) -> Result<Sample, SongError> {
    let name = reader.read_str(22)?;
    let length = reader.read_u16()? * 2;

    let raw_finetune = reader.read_u8()? & 0x0F;
    let value = (raw_finetune & 0b0111) as i8;
    let sign = (raw_finetune & 0b1000) != 0;

    let finetune = if sign { -(value as i8) } else { value as i8 };
    let volume = reader.read_u8()?;

    let repeat_offset = reader.read_u16()? * 2;
    let mut repeat_length = reader.read_u16()? * 2;

    // No idea why its saved as a 1 when not repeated
    if repeat_length == 2 {
        repeat_length = 0;
    }

    Ok(Sample {
        name,
        length,
        finetune,
        volume,
        repeat_offset,
        repeat_length,
    })
}

fn detect_sample_count(format_tag: &str) -> usize {
    // List of known 31-sample MOD format tags
    const KNOWN_TAGS: [&str; 30] = [
        "M.K.", "M!K!", "FLT4", "FLT8", "CD81", "2CHN", "4CHN", "6CHN", "8CHN", "10CH", "12CH",
        "14CH", "16CH", "18CH", "20CH", "22CH", "24CH", "26CH", "28CH", "30CH", "32CH", "11CH",
        "13CH", "15CH", // TakeTracker
        "TDZ1", "TDZ2", "TDZ3", "5CHN", "7CHN", "9CHN",
    ];

    if KNOWN_TAGS.contains(&format_tag) {
        31
    } else {
        // Fallback: check if all characters are printable ASCII
        let printable_ascii = format_tag
            .chars()
            .all(|c| c != '\0' && (32..=126).contains(&(c as u8)));

        if printable_ascii {
            31
        } else {
            15
        }
    }
}

fn guess_channel_count(file_size: usize, sample_meta: &Vec<Sample>, pattern_count: u8) -> u8 {
    // Guess channel count from calculating the size of the pattern data
    // To get the amount of patterns, we find the highest pattern played + 1 from the pattern table
    // A pattern consists of 64 lines, and each note is 4 bytes, each line has nr_channels of bytes

    // Size of a single pattern = 64 * 4 * nr_channels
    // Re-arranged: nr_channels = pattern_size / (64 * 4)

    // To get the size of one pattern, we take the total pattern data size and divide by nr_patterns
    // Then we can substitute in the equation above

    let sample_count = sample_meta.len();

    // Pattern table + Title + End pos + Samples played
    let song_metadata_size: u8 = 128 + 20 + 2;
    // The metadata for a single sample is 30 bytes
    let sample_meta_size: usize = 30 * sample_count;
    // Older 15 sample mods don't have a format tag
    let format_size = if sample_count == 31 { 4 } else { 0 };

    let mut sample_pcm_size: u32 = 0;
    for index in 0..sample_count {
        sample_pcm_size += sample_meta[index].length as u32;
    }

    let pattern_data_left: u32 = file_size as u32
        - song_metadata_size as u32
        - sample_meta_size as u32
        - format_size
        - sample_pcm_size;

    return ((pattern_data_left / pattern_count as u32) / (64 * 4)) as u8;
}

// This takes in 4 parameters because we may need to "guess" the amount of channels if we can't derive it from the tag
fn identify_format_and_channels(
    tag: &str,
    file_size: usize,
    sample_metadata: &Vec<Sample>,
    pattern_count: u8,
) -> (u8, Tracker) {
    match tag {
        // Very common tags
        "M.K." | "M!K!" => (4, Tracker::ProTracker),
        "FLT4" => (4, Tracker::Startrekker),
        "FLT8" => (8, Tracker::Startrekker),

        // Less common tags
        "CD81" => (8, Tracker::Falcon),
        "OCTA" => (8, Tracker::Oktalyzer),

        // Generic channel tags
        "2CHN" => (2, Tracker::FastTracker),

        // 4CHN may also be used by NoiseTracker or ProTracker
        "4CHN" => (4, Tracker::FastOrNoiseTracker),
        "6CHN" => (6, Tracker::FastTracker),
        "8CHN" => (8, Tracker::FastTracker),

        // TakeTracker
        "TDZ1" => (1, Tracker::TakeTracker),
        "TDZ2" => (2, Tracker::TakeTracker),
        "TDZ3" => (3, Tracker::TakeTracker),
        "5CHN" => (5, Tracker::TakeTracker),
        "7CHN" => (7, Tracker::TakeTracker),
        "9CHN" => (9, Tracker::TakeTracker),
        "11CH" => (11, Tracker::TakeTracker),
        "13CH" => (13, Tracker::TakeTracker),
        "15CH" => (15, Tracker::TakeTracker),

        _ => {
            // Detect yyCH FastTracker mods
            if tag.ends_with("CH") {
                if let Ok(yy) = tag[0..2].parse::<u8>() {
                    if (10..=32).contains(&yy) && yy % 2 == 0 {
                        return (yy, Tracker::FastTracker);
                    }
                }
            }

            return (
                guess_channel_count(file_size, sample_metadata, pattern_count),
                Tracker::Generic,
            );
        }
    }
}

fn read_note(reader: &mut ByteReader) -> Result<song::Note, SongError> {
    //              Byte  1   Byte  2   Byte  3   Byte 4
    //              --------- --------- --------- ---------
    //              7654-3210 7654-3210 7654-3210 7654-3210
    //              wwww XXXX xxxxxxxxx yyyy ZZZZ zzzzzzzzz
    //
    //                  wwwwyyyy ( 8 bits) : sample number
    //              XXXXxxxxxxxx (12 bits) : sample 'period'
    //              ZZZZzzzzzzzz (12 bits) : effect and argument

    let bytes = reader.read_bytes(4)?;

    let sample: u8 = (bytes[0] & 0xF0) | ((bytes[2] & 0xF0) >> 4);
    let period: u16 = (((bytes[0] & 0x0F) as u16) << 8) | (bytes[1] as u16);
    let effect: u8 = bytes[2] & 0x0F;
    let argument: u8 = bytes[3];

    Ok(song::Note {
        sample,
        period,
        effect,
        argument,
    })
}

fn read_pattern(reader: &mut ByteReader, channel_count: u8) -> Result<song::Pattern, SongError> {
    let mut pattern: song::Pattern = Vec::with_capacity(64);

    // Read all the lines
    for _ in 1..=64 {
        let mut line: song::Line = Vec::with_capacity(channel_count as usize);

        // Read all the notes
        for _ in 1..=channel_count {
            line.push(read_note(reader)?);
        }

        pattern.push(line);
    }

    Ok(pattern)
}

pub fn parse(data: Vec<u8>) -> Result<Song, SongError> {
    let mut reader = ByteReader::new(&data, Encoding::BigEndian);

    // Ensure there's atleast 1080 bytes before hand, this isn't enough, but doesn't hurt to check prematurely
    if reader.seek(1080).is_err() {
        return Err(SongError::Read("Invalid or corrupted module".into()));
    }

    let format = reader
        .read_str(4)
        .unwrap_or_else(|_| String::from("\0\0\0\0"));

    // Check if all characters in the format string are in the printable ASCII range (32–126)
    let sample_count = detect_sample_count(&format);

    // Unwrap because we know we can safely jump to offset 0
    reader.seek(0).unwrap();

    let title = reader.read_str(20)?;
    let mut sample_metadata: Vec<Sample> = Vec::with_capacity(sample_count);

    for index in 0..sample_count {
        let sample = read_sample(&mut reader)?;

        #[cfg(debug_assertions)]
        if sample.name.len() > 0 {
            let read_sample = format!(
                "Read sample {} ({}/{}) len: {}",
                &sample.name,
                index + 1,
                sample_count,
                sample.length
            );

            dbg!(read_sample);
        }

        sample_metadata.push(sample);
    }

    // Patterns played, we can skip this (I think?)
    reader.read_u8()?;
    let end_jmp_pos = reader.read_i8()?;

    let pattern_table = reader
        .read_bytes(128)
        .map_err(|_| SongError::Read("Failed to read pattern table".into()))?
        .to_vec();

    let pattern_count = pattern_table.iter().max().unwrap_or(&0) + 1;

    // Skip reading the format tag, we've already read it above
    if sample_count == 31 {
        reader.seek(reader.position() + 4)?;
    }

    let (channel_count, tracker) =
        identify_format_and_channels(&format, data.len(), &sample_metadata, pattern_count);

    let mut patterns: Vec<song::Pattern> = Vec::with_capacity(pattern_count as usize);
    for _ in 0..pattern_count {
        patterns.push(read_pattern(&mut reader, channel_count)?);
    }

    let mut samples: Vec<song::PCMData> = Vec::with_capacity(sample_count);
    for i in 0..sample_count {
        let length = sample_metadata.get(i).unwrap().length as usize;

        let sample = reader
            .read_bytes(length)
            .map_err(|_| SongError::Read("Failed to read sample data".into()))?
            .to_vec();

        samples.push(song::PCMData::U8(sample));
    }

    if matches!(tracker, Tracker::ProTracker) {
        let pat = patterns.get(0).unwrap();

        let mut lineno = 0;
        pat.iter().for_each(|line| {
            let line_str = line
                .iter()
                .map(|note| {
                    let finetune = sample_metadata.get(note.sample as usize).unwrap().finetune;

                    let pnote = tracker::protracker_period_to_note(note.period, finetune);

                    pnote.unwrap_or(String::from("---"))
                })
                .collect::<Vec<_>>()
                .join(" ");
            println!("{}: {}", lineno, line_str);

            lineno += 1;
        });
    }

    let metadata = song::SongMetadata {
        name: title,
        samples: sample_metadata,

        pattern_table,

        pattern_count: pattern_count,
        channel_count,

        end_jump: end_jmp_pos,
        format,

        tracker,
    };

    Ok(Song {
        metadata,
        patterns,
        samples,
    })
}

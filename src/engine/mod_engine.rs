use super::TrackerEngine;
use crate::tracker;
use crate::{song, Song};

pub struct ModEngine {
    pub song: Song,
    pub current_row: usize,
    pub current_pattern: usize,

    pub samples_since_tick: usize,
    pub samples_per_tick: usize,

    // Current tick
    pub tick: u8,
    // Ticks per row (How many ticks before advancing to next row)
    pub speed: u8,
    // BPM (determines how long a tick lasts)
    pub tempo: u16,

    pub channels: Vec<ChannelState>, // One per channel (e.g. 4)

    // Output device
    pub sample_rate: u32,
    pub channel_count: u16,
}

#[derive(Clone, Debug)]
pub struct ChannelState {
    pub sample_index: usize,
    pub volume: u8,
    pub period: u16,
    pub effect: u8,
    pub effect_arg: u8,

    // Hard left = 0; Middle = 128; Hard right = 255
    pub panning: u8,

    pub position_in_sample: f32, // For mixing audio
    pub sample_step: f32,        // Based on period + sample_rate

    pub base_period: u16,

    pub repeat_offset: u16,
    pub repeat_length: u16,

    pub arp_counter: u8,
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            sample_index: 0,
            volume: 64,
            period: 0,
            effect: 0,
            effect_arg: 0,
            panning: 128, //192,

            repeat_offset: 0,
            repeat_length: 0,

            position_in_sample: 0.0,
            sample_step: 0.0,

            base_period: 0,
            arp_counter: 0,
        }
    }
}

impl TrackerEngine for ModEngine {
    fn samples_per_tick(&self) -> usize {
        self.samples_per_tick
    }

    fn set_samples_per_tick(&mut self, value: usize) {
        self.samples_per_tick = value;
    }

    fn samples_since_tick(&self) -> usize {
        self.samples_since_tick
    }

    fn set_samples_since_tick(&mut self, value: usize) {
        self.samples_since_tick = value;
    }

    fn get_audio_buffer(&mut self, buffer: &mut [f32]) {
        let num_channels = self.channel_count as usize;
        let samples_per_buffer = buffer.len() / num_channels as usize;

        // For each output sample (frame)
        for i in 0..samples_per_buffer {
            let mut left = 0.0f32;
            let mut right = 0.0f32;

            // Mix all tracker channels
            for (_, channel) in self.channels.iter_mut().enumerate() {
                // Get sample data for this channel
                let sample = match &self.song.samples[channel.sample_index] {
                    song::PCMData::I8(data) => data,
                    _ => continue,
                };

                // Fetch sample value (simple nearest-neighbor, can use interpolation for quality)
                let pos = channel.position_in_sample as usize;
                let sample_val = if pos < sample.len() {
                    // Convert 8-bit sample [-128,127] to [-1.0,1.0]
                    sample[pos] as f32 / 128.0
                } else {
                    0.0
                };

                // Apply volume (0..64)
                let vol = channel.volume.min(64) as f32 / 64.0;
                let out_val = sample_val * vol;

                let pan = channel.panning as f32 / 255.0;
                left += out_val * (1.0 - pan);
                right += out_val * pan;

                channel.position_in_sample += channel.sample_step;
            }

            for ch in 0..num_channels {
                buffer[i * num_channels + ch] = match ch {
                    0 => left.clamp(-1.0, 1.0),  // Left
                    1 => right.clamp(-1.0, 1.0), // Right
                    _ => 0.0,                    // Silence for other channels
                };
            }
        }
    }

    fn is_finished(&self) -> bool {
        return self.current_row == 64;
    }

    fn next_tick(&mut self) {
        let pattern = &self.song.patterns[self.current_pattern];
        let line = &pattern[self.current_row];

        if self.tick == 0 {
            print_line(pattern, &self.song.metadata.samples, self.current_row);
        }

        for (index, channel) in self.channels.iter_mut().enumerate() {
            if self.tick == 0 {
                let note = line.get(index).unwrap();
                let new_period = note.period;
                let mut new_sample_index = note.sample as usize;

                if new_period != 0 {
                    if new_sample_index == 0 {
                        new_sample_index = channel.sample_index;
                    }

                    channel.position_in_sample = 0.0;
                    channel.base_period = new_period;
                    channel.arp_counter = 0;

                    // Set repeat info from sample metadata
                    if new_sample_index > 0 {
                        let sample_meta = &self.song.metadata.samples[new_sample_index - 1];
                        channel.repeat_offset = sample_meta.repeat_offset;
                        channel.repeat_length = sample_meta.repeat_length;
                        channel.volume = sample_meta.volume.min(64);
                        channel.sample_index = new_sample_index - 1;
                    }

                    channel.effect = note.effect;
                    channel.effect_arg = note.argument;
                    channel.period = channel.base_period;
                    channel.sample_step = 0.0;
                } else if note.sample != 0 {
                    // Instrument only: update instrument, but do NOT reset position or period
                    let sample_meta = &self.song.metadata.samples[new_sample_index - 1];
                    channel.repeat_offset = sample_meta.repeat_offset;
                    channel.repeat_length = sample_meta.repeat_length;
                    channel.volume = sample_meta.volume.min(64);

                    channel.sample_index = new_sample_index - 1;
                    channel.effect = note.effect;
                    channel.effect_arg = note.argument;
                } else {
                    // Effect only: just update effect/argument
                    channel.effect = note.effect;
                    channel.effect_arg = note.argument;
                }
            }

            process_effects(channel, self.tick);

            // Amiga PAL clock for MOD: 7093789.2 Hz
            if channel.period != 0 {
                let freq = 7093789.2 / (channel.period as f32 * 2.0);

                channel.sample_step = freq / self.sample_rate as f32;
            }
        }

        self.tick += 1;
        if self.tick >= self.speed {
            self.tick = 0;
            self.current_row += 1;

            if self.current_row == 64 {
                self.current_row = 0;
                self.current_pattern += 1;

                // Optionally, loop back to the first pattern if at the end of the song
                if self.current_pattern >= self.song.patterns.len() {
                    self.current_pattern = 0;
                }

                println!("Playing pattern: {}", self.current_pattern);
            }
        }
    }
}

impl ModEngine {
    pub fn new(
        song: Song,
        sample_rate: cpal::SampleRate,
        channel_count: cpal::ChannelCount,
    ) -> Self {
        let mut channels = Vec::with_capacity(song.metadata.channel_count as usize);
        for _ in 0..song.metadata.channel_count {
            channels.push(ChannelState::default());
        }

        let tempo = 125;
        let tick_duration = 2.5 / tempo as f32;
        let samples_per_tick = (sample_rate.0 as f32 * tick_duration) as usize;

        ModEngine {
            song,
            current_row: 0,
            current_pattern: 0,

            tick: 0,
            speed: 8,
            tempo,

            channel_count,

            samples_per_tick,
            samples_since_tick: 0,

            channels,
            sample_rate: 48000,
        }
    }
}

fn print_line(pattern: &song::Pattern, sample_metadata: &Vec<song::Sample>, lineno: usize) {
    let line = pattern.get(lineno).unwrap();
    let line_str = line
        .iter()
        .map(|note| {
            let finetune = sample_metadata.get(note.sample as usize).unwrap().finetune;
            let pnote = tracker::protracker_period_to_note(note.period, finetune);

            pnote.unwrap_or(String::from("---"))
        })
        .collect::<Vec<_>>()
        .join(" ");

    println!("{:02}: {}", lineno, line_str);
}

fn process_effects(channel: &mut ChannelState, tick: u8) {
    match channel.effect {
        // 0x0: Arpeggio
        0x0 => {
            if tick > 0 {
                let x = (channel.effect_arg & 0xF0) >> 4;
                let y = channel.effect_arg & 0x0F;
                if x == 0 && y == 0 {
                    return;
                }

                if tick == 0 || tick == 1 {
                    // On tick 0 and 1, play the actual note (no change)
                    if tick == 1 {
                        channel.arp_counter = 1;
                    } else {
                        channel.arp_counter = 0;
                    }
                    channel.period = channel.base_period;
                } else {
                    let offset = match channel.arp_counter % 3 {
                        1 => x,
                        2 => y,
                        _ => 0,
                    };
                    channel.period = channel.base_period.saturating_add(offset as u16);
                    channel.arp_counter = channel.arp_counter.wrapping_add(1);
                }
            }
        }

        // 0x1: Portamento Up
        0x1 => {
            if tick > 0 {
                let step = channel.effect_arg as u16;
                channel.period = channel.base_period.saturating_sub(step);
                channel.base_period = channel.period;
            }
        }

        // 0x2: Portamento Down
        0x2 => {
            if tick > 0 {
                let step = channel.effect_arg as u16;
                channel.period = channel.base_period.saturating_add(step);
                channel.base_period = channel.period;
            }
        }

        // 0x3: Tone Portamento
        0x3 => {
            if tick > 0 {
                let target_period = channel.period;
                let step = channel.effect_arg as u16;
                if channel.period > target_period {
                    channel.period = channel.base_period.saturating_sub(step);
                } else if channel.period < target_period {
                    channel.period = channel.base_period.saturating_add(step);
                }
            }
        }

        // 0x4: Vibrato
        0x4 => {
            if tick > 0 {
                let speed = (channel.effect_arg & 0xF0) >> 4; // High nibble
                let depth = channel.effect_arg & 0x0F; // Low nibble
                                                       // TODO: Implement vibrato logic using a sine wave table
            }
        }

        // 0xA: Volume Slide
        0xA => {
            if tick > 0 {
                let slide_up = (channel.effect_arg & 0xF0) >> 4; // High nibble
                let slide_down = channel.effect_arg & 0x0F; // Low nibble
                if slide_up > 0 {
                    channel.volume = channel.volume.saturating_add(slide_up);
                } else if slide_down > 0 {
                    channel.volume = channel.volume.saturating_sub(slide_down);
                }
            }
        }

        // 0xB: Position Jump
        0xB => {
            if tick == 0 {
                // Position Jump (Bxx): Jumps to a specific pattern
                // TODO: Implement position jump logic in the engine
            }
        }

        // 0xC: Set Volume
        0xC => {
            if tick == 0 {
                // Set Volume (Cxx): Sets the volume to xx
                channel.volume = channel.effect_arg.min(64); // Clamp to max volume of 64
            }
        }

        // 0xD: Pattern Break
        0xD => {
            if tick == 0 {
                // Pattern Break (Dxx): Jumps to a specific row in the next pattern
                // TODO: Implement pattern break logic in the engine
            }
        }

        // 0xE: Extended Effects
        0xE => {
            let sub_effect = (channel.effect_arg & 0xF0) >> 4; // High nibble
            let sub_arg = channel.effect_arg & 0x0F; // Low nibble
            match sub_effect {
                // E1x: Fine Portamento Up
                0x1 => {
                    if tick == 0 {
                        channel.period = channel.base_period.saturating_sub(sub_arg as u16);
                    }
                }
                // E2x: Fine Portamento Down
                0x2 => {
                    if tick == 0 {
                        channel.period = channel.base_period.saturating_add(sub_arg as u16);
                    }
                }
                // E9x: Retrigger Note
                0x9 => {
                    if tick % sub_arg == 0 {
                        // TODO: Retrigger the note
                    }
                }
                _ => {}
            }
        }

        // 0xF: Set Speed/Tempo
        0xF => {
            if tick == 0 {
                if channel.effect_arg <= 0x1F {
                    // Set speed (ticks per row)
                    // TODO: Update the engine's speed
                } else {
                    // Set tempo (BPM)
                    // TODO: Update the engine's tempo
                }
            }
        }

        _ => {
            // Unknown effect
            //eprintln!("Unknown effect: {:X}", channel.effect);
        }
    }
}

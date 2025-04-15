use std::time::Duration;

use super::TrackerEngine;
use crate::tracker::{self, Tracker};
use crate::{song, Song};

pub struct ModEngine<'a> {
    pub song: &'a Song,
    pub current_row: usize,
    pub current_pattern: usize,
    // Current tick
    pub tick: u8,
    // Ticks per row (How many ticks before advancing to next row)
    pub speed: u8,
    // BPM (determines how long a tick lasts)
    pub tempo: u16,

    pub channels: Vec<ChannelState>, // One per channel (e.g. 4)
    pub sample_rate: u32,
}

#[derive(Clone, Debug)]
pub struct ChannelState {
    pub sample_index: usize,
    pub volume: u8,
    pub period: u16,
    pub effect: u8,
    pub effect_arg: u8,

    pub position_in_sample: f32, // For mixing audio
    pub sample_step: f32,        // Based on period + sample_rate
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState {
            sample_index: 0,
            volume: 64,
            period: 0,
            effect: 0,
            effect_arg: 0,

            position_in_sample: 0.0,
            sample_step: 0.0,
        }
    }
}

impl TrackerEngine for ModEngine<'_> {
    fn get_audio_buffer(&mut self, buffer: &mut [f32]) {
        todo!();
    }

    fn sleep_duration(&self) -> std::time::Duration {
        return Duration::from_secs_f32(2.5 / self.tempo as f32);
    }

    fn is_finished(&self) -> bool {
        return self.current_row == 64;
    }

    fn next_tick(&mut self) {
        // On tick 0 we read all the notes and trigger effects
        let pattern = self.song.patterns.get(self.current_pattern).unwrap();
        let line = pattern.get(self.current_row).unwrap();

        if self.tick == 0 {
          print_line(pattern, &self.song.metadata.samples, self.current_row);
        }

        for (index, channel) in self.channels.iter_mut().enumerate() {
            if self.tick == 0 {
                let note = line.get(index).unwrap();
                channel.effect = note.effect;
                channel.effect_arg = note.argument;
                channel.period = note.period;
                channel.sample_index = note.sample as usize;

                channel.position_in_sample = 0.0;
                channel.sample_step = 0.0;
            }

            process_effects(channel, self.tick);
        }

        self.tick += 1;
        if self.tick > self.speed {
            self.tick = 0;
            self.current_row += 1;
        }
    }
}

impl<'a> ModEngine<'a> {
    pub fn new(song: &'a Song) -> Self {
        let mut channels = Vec::with_capacity(song.metadata.channel_count as usize);
        for _ in 0..song.metadata.channel_count {
            channels.push(ChannelState::default());
        }

        ModEngine {
            song,
            current_row: 0,
            current_pattern: 0,
            tick: 0,
            speed: 6,
            tempo: 125,
            channels,
            sample_rate: 44100,
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

                // Cycle between base note, base note + x, and base note + y
                let offset = match tick % 3 {
                    1 => x,
                    2 => y,
                    _ => 0,
                };
                channel.period = channel.period.saturating_add(offset as u16);
            }
        }

        // 0x1: Portamento Up
        0x1 => {
            if tick > 0 {
                let step = channel.effect_arg as u16;
                channel.period = channel.period.saturating_sub(step);
            }
        }

        // 0x2: Portamento Down
        0x2 => {
            if tick > 0 {
                let step = channel.effect_arg as u16;
                channel.period = channel.period.saturating_add(step);
            }
        }

        // 0x3: Tone Portamento
        0x3 => {
            if tick > 0 {
                let target_period = channel.period; // TODO: Set this based on the note
                let step = channel.effect_arg as u16;
                if channel.period > target_period {
                    channel.period = channel.period.saturating_sub(step);
                } else if channel.period < target_period {
                    channel.period = channel.period.saturating_add(step);
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
                        channel.period = channel.period.saturating_sub(sub_arg as u16);
                    }
                }
                // E2x: Fine Portamento Down
                0x2 => {
                    if tick == 0 {
                        channel.period = channel.period.saturating_add(sub_arg as u16);
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

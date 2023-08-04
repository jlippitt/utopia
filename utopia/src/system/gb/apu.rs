use crate::AudioQueue;
use noise::Noise;
use pulse::Pulse;
use tracing::{debug, warn};
use wave::Wave;

mod component;
mod noise;
mod pulse;
mod wave;

const CYCLES_PER_SECOND: u64 = 1048576;
const CLOCK_SHIFT: u32 = 16;
const SAMPLE_PERIOD: u64 = (CYCLES_PER_SECOND << CLOCK_SHIFT) / Apu::SAMPLE_RATE;
const SAMPLES_PER_CYCLE: u64 = 1 << CLOCK_SHIFT;

// 0-15 channel output, 4 channels, 0-7 volume level
const MAX_OUTPUT_VALUE: f32 = 15.0 * 4.0 * 7.0;

#[derive(Clone, Default)]
struct Channel {
    enabled: [bool; 4],
    volume: u8,
}

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    wave: Wave,
    noise: Noise,
    divider: u64,
    sample_clock: u64,
    power: bool,
    channels: [Channel; 2],
    audio_queue: AudioQueue,
}

impl Apu {
    pub const SAMPLE_RATE: u64 = 44100;

    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            wave: Wave::new(),
            noise: Noise::new(),
            divider: 0,
            sample_clock: 0,
            power: false,
            channels: Default::default(),
            audio_queue: AudioQueue::new(),
        }
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn read(&mut self, address: u8) -> u8 {
        match address {
            0x24 => (self.channels[0].volume << 4) | self.channels[1].volume,
            0x25 => {
                let mut value = 0;
                value |= if self.channels[0].enabled[0] { 0x10 } else { 0 };
                value |= if self.channels[0].enabled[1] { 0x20 } else { 0 };
                value |= if self.channels[0].enabled[2] { 0x40 } else { 0 };
                value |= if self.channels[0].enabled[3] { 0x80 } else { 0 };
                value |= if self.channels[1].enabled[0] { 0x01 } else { 0 };
                value |= if self.channels[1].enabled[1] { 0x02 } else { 0 };
                value |= if self.channels[1].enabled[2] { 0x04 } else { 0 };
                value |= if self.channels[1].enabled[3] { 0x08 } else { 0 };
                value
            }
            _ => {
                warn!("APU register read not yet implemented: {:02X}", address);
                0
            }
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x10..=0x14 => self.pulse1.write(address - 0x10, value),
            0x15..=0x19 => self.pulse2.write(address - 0x15, value),
            0x1a..=0x1e => self.wave.write_register(address - 0x1a, value),
            0x1f..=0x23 => self.noise.write(address - 0x1f, value),
            0x24 => {
                self.channels[0].volume = (value >> 4) & 7;
                self.channels[1].volume = value & 7;
            }
            0x25 => {
                self.channels[0].enabled[0] = (value & 0x10) != 0;
                self.channels[0].enabled[1] = (value & 0x20) != 0;
                self.channels[0].enabled[2] = (value & 0x40) != 0;
                self.channels[0].enabled[3] = (value & 0x80) != 0;
                self.channels[1].enabled[0] = (value & 0x01) != 0;
                self.channels[1].enabled[1] = (value & 0x02) != 0;
                self.channels[1].enabled[2] = (value & 0x04) != 0;
                self.channels[1].enabled[3] = (value & 0x08) != 0;
            }
            0x26 => self.power = (value & 0x80) != 0,
            0x30..=0x3f => self.wave.write_ram(address as usize - 0x30, value),
            _ => warn!(
                "APU register write not yet implemented: {:02X} <= {:02X}",
                address, value
            ),
        }
    }

    pub fn step(&mut self) {
        self.pulse1.step();
        self.pulse2.step();
        self.wave.step();
        self.noise.step();

        self.sample_clock += SAMPLES_PER_CYCLE;

        if self.sample_clock < SAMPLE_PERIOD {
            return;
        }

        self.sample_clock -= SAMPLE_PERIOD;

        let output = if self.power {
            let left = self.channel_output(0);
            let right = self.channel_output(1);
            (left, right)
        } else {
            (0.0, 0.0)
        };

        self.audio_queue.push_back(output);
    }

    pub fn on_divider_clock(&mut self) {
        debug!("APU Divider Clock");
        self.divider += 1;
        self.pulse1.on_divider_clock(self.divider);
        self.pulse2.on_divider_clock(self.divider);
        self.wave.on_divider_clock(self.divider);
        self.noise.on_divider_clock(self.divider);
    }

    fn channel_output(&self, index: usize) -> f32 {
        let channel = &self.channels[index];
        let mut output = 0;

        if channel.enabled[0] {
            output += self.pulse1.output();
        }

        if channel.enabled[1] {
            output += self.pulse2.output();
        }

        if channel.enabled[2] {
            output += self.wave.output();
        }

        if channel.enabled[3] {
            output += self.noise.output();
        }

        ((channel.volume as f32 * output as f32) / MAX_OUTPUT_VALUE) - 0.5
    }
}

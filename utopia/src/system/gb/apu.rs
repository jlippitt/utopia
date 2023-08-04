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

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    wave: Wave,
    noise: Noise,
    divider: u64,
    sample_clock: u64,
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
            audio_queue: AudioQueue::new(),
        }
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn read(&mut self, address: u8) -> u8 {
        warn!("APU register read not yet implemented: {:02X}", address);
        0
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x10..=0x14 => self.pulse1.write(address - 0x10, value),
            0x15..=0x19 => self.pulse2.write(address - 0x15, value),
            0x1a..=0x1e => self.wave.write_register(address - 0x1a, value),
            0x1f..=0x23 => self.noise.write(address - 0x1f, value),
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

        if self.sample_clock >= SAMPLE_PERIOD {
            self.sample_clock -= SAMPLE_PERIOD;

            let raw_output = self.pulse1.output()
                + self.pulse2.output()
                + self.wave.output()
                + self.noise.output();

            let output = (raw_output as f32) / 60.0;

            self.audio_queue.push_back((output, output));
        }
    }

    pub fn on_divider_clock(&mut self) {
        debug!("APU Divider Clock");
        self.divider += 1;
        self.pulse1.on_divider_clock(self.divider);
        self.pulse2.on_divider_clock(self.divider);
        self.wave.on_divider_clock(self.divider);
        self.noise.on_divider_clock(self.divider);
    }
}

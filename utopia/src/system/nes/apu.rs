use crate::AudioQueue;
use dmc::Dmc;
use frame::FrameCounter;
use noise::Noise;
use pulse::Pulse;
use tracing::warn;
use triangle::Triangle;

mod component;
mod dmc;
mod frame;
mod noise;
mod pulse;
mod triangle;

const CYCLES_PER_SECOND: u64 = 1789773;
const CLOCK_SHIFT: u32 = 16;
const SAMPLE_PERIOD: u64 = (CYCLES_PER_SECOND << CLOCK_SHIFT) / Apu::SAMPLE_RATE;
const SAMPLES_PER_CYCLE: u64 = 1 << CLOCK_SHIFT;

const PULSE_TABLE_SIZE: usize = 31;
const TND_TABLE_SIZE: usize = 203;

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    frame_counter: FrameCounter,
    sample_clock: u64,
    audio_queue: AudioQueue,
    pulse_table: [f32; PULSE_TABLE_SIZE],
    tnd_table: [f32; TND_TABLE_SIZE],
}

impl Apu {
    pub const SAMPLE_RATE: u64 = 44100;

    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            frame_counter: FrameCounter::new(),
            sample_clock: 0,
            audio_queue: AudioQueue::new(),
            pulse_table: create_pulse_table(),
            tnd_table: create_tnd_table(),
        }
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn read_register(&mut self, address: u16, prev_value: u8) -> u8 {
        match address & 0x1f {
            0x15 => {
                let mut value = prev_value & 0x20;
                value |= if self.pulse1.enabled() { 0x01 } else { 0 };
                value |= if self.pulse2.enabled() { 0x02 } else { 0 };
                value |= if self.triangle.enabled() { 0x04 } else { 0 };
                value |= if self.noise.enabled() { 0x08 } else { 0 };
                value |= if self.dmc.enabled() { 0x10 } else { 0 };
                // TODO: IRQ status
                value
            }
            _ => {
                warn!("Unmapped APU Read: {:04X}", address);
                prev_value
            }
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address & 0x1f {
            0x00..=0x03 => self.pulse1.write(address, value),
            0x04..=0x07 => self.pulse2.write(address, value),
            0x08..=0x0b => self.triangle.write(address, value),
            0x0c..=0x0f => self.noise.write(address, value),
            0x10..=0x14 => self.dmc.write(address, value),
            0x15 => {
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
                self.triangle.set_enabled((value & 0x04) != 0);
                self.noise.set_enabled((value & 0x08) != 0);
                self.dmc.set_enabled((value & 0x10) != 0);
            }
            0x17 => {
                self.frame_counter.set_mode((value & 0x80) != 0);
                // TODO: Frame IRQ
            }
            _ => warn!("Unmapped APU Write: {:04X} <= {:02X}", address, value),
        }
    }

    pub fn step(&mut self) {
        self.pulse1.step();
        self.pulse2.step();
        self.triangle.step();
        self.noise.step();
        self.dmc.step();

        if let Some(event) = self.frame_counter.step() {
            self.pulse1.on_frame_event(event);
            self.pulse2.on_frame_event(event);
            self.triangle.on_frame_event(event);
            self.noise.on_frame_event(event);
        }

        self.sample_clock += SAMPLES_PER_CYCLE;

        if self.sample_clock >= SAMPLE_PERIOD {
            self.sample_clock -= SAMPLE_PERIOD;

            let pulse = self.pulse1.sample() + self.pulse2.sample();
            let tnd = self.triangle.sample() * 3 + self.noise.sample() * 2 + self.dmc.sample();
            let sample = self.pulse_table[pulse as usize] + self.tnd_table[tnd as usize];

            self.audio_queue.push_back((sample, sample));
        }
    }
}

fn create_pulse_table() -> [f32; PULSE_TABLE_SIZE] {
    let mut table = [0.0; PULSE_TABLE_SIZE];

    for (index, entry) in table.iter_mut().enumerate() {
        *entry = 95.52 / (8128.0 / (index as f32) + 100.0);
    }

    table
}

fn create_tnd_table() -> [f32; TND_TABLE_SIZE] {
    let mut table = [0.0; TND_TABLE_SIZE];

    for (index, entry) in table.iter_mut().enumerate() {
        *entry = 163.67 / (24329.0 / (index as f32) + 100.0)
    }

    table
}

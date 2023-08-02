use crate::AudioQueue;
use frame::FrameCounter;
use pulse::Pulse;
use tracing::warn;

mod component;
mod frame;
mod pulse;

const CYCLES_PER_SECOND: u64 = 1789773;
const CLOCK_SHIFT: u32 = 16;
const SAMPLE_PERIOD: u64 = (CYCLES_PER_SECOND << CLOCK_SHIFT) / Apu::SAMPLE_RATE;
const SAMPLES_PER_CYCLE: u64 = 1 << CLOCK_SHIFT;

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    frame_counter: FrameCounter,
    sample_clock: u64,
    total_samples: u64,
    audio_queue: AudioQueue,
    pulse_table: [f32; 31],
}

impl Apu {
    pub const SAMPLE_RATE: u64 = 44100;

    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            frame_counter: FrameCounter::new(),
            sample_clock: 0,
            total_samples: 0,
            audio_queue: AudioQueue::new(),
            pulse_table: create_pulse_table(),
        }
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    pub fn read_register(&mut self, address: u16, prev_value: u8) -> u8 {
        match address & 0x1f {
            0x15 => {
                let mut value = prev_value & 0x20;
                value |= if self.pulse1.enabled() { 0x01 } else { 0 };
                value |= if self.pulse2.enabled() { 0x02 } else { 0 };
                // TODO: Triangle
                // TODO: Noise
                // TODO: DMC
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
            0x15 => {
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
                // TODO: Triangle
                // TODO: Noise
                // TODO: DMC
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

        if let Some(event) = self.frame_counter.step() {
            self.pulse1.on_frame_event(event);
            self.pulse2.on_frame_event(event);
        }

        self.sample_clock += SAMPLES_PER_CYCLE;

        if self.sample_clock >= SAMPLE_PERIOD {
            self.sample_clock -= SAMPLE_PERIOD;

            let pulse = self.pulse1.sample() + self.pulse2.sample();

            let sample = self.pulse_table[pulse as usize];

            self.audio_queue.push_back((sample, sample));
            self.total_samples += 1;
        }
    }
}

fn create_pulse_table() -> [f32; 31] {
    let mut table = [0.0; 31];

    for (index, entry) in table.iter_mut().enumerate() {
        *entry = 95.52 / (8128.0 / (index as f32) + 100.0);
    }

    table
}

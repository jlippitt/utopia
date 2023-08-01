use crate::AudioQueue;
use frame::FrameCounter;
use tracing::warn;

mod frame;

const CYCLES_PER_SECOND: u64 = 1789773;
const CLOCK_SHIFT: u32 = 16;
const SAMPLE_PERIOD: u64 = (CYCLES_PER_SECOND << CLOCK_SHIFT) / Apu::SAMPLE_RATE;
const SAMPLES_PER_CYCLE: u64 = 1 << CLOCK_SHIFT;

pub struct Apu {
    frame_counter: FrameCounter,
    sample_clock: u64,
    total_samples: u64,
    audio_queue: AudioQueue,
}

impl Apu {
    pub const SAMPLE_RATE: u64 = 44100;

    pub fn new() -> Self {
        Self {
            frame_counter: FrameCounter::new(),
            sample_clock: 0,
            total_samples: 0,
            audio_queue: AudioQueue::new(),
        }
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address & 0x1f {
            0x17 => {
                self.frame_counter.set_mode((value & 0x80) != 0);
                // TODO: Frame IRQ
            }
            _ => warn!("Unmapped APU Write: {:04X} <= {:02X}", address, value),
        }
    }

    pub fn step(&mut self) {
        let _frame = self.frame_counter.step();

        self.sample_clock += SAMPLES_PER_CYCLE;

        if self.sample_clock >= SAMPLE_PERIOD {
            self.sample_clock -= SAMPLE_PERIOD;
            self.audio_queue.push_back((0, 0));
            self.total_samples += 1;
        }
    }
}

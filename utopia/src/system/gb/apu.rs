use crate::AudioQueue;

const CYCLES_PER_SECOND: u64 = 1048576;
const CLOCK_SHIFT: u32 = 16;
const SAMPLE_PERIOD: u64 = (CYCLES_PER_SECOND << CLOCK_SHIFT) / Apu::SAMPLE_RATE;
const SAMPLES_PER_CYCLE: u64 = 1 << CLOCK_SHIFT;

pub struct Apu {
    sample_clock: u64,
    audio_queue: AudioQueue,
}

impl Apu {
    pub const SAMPLE_RATE: u64 = 44100;

    pub fn new() -> Self {
        Self {
            sample_clock: 0,
            audio_queue: AudioQueue::new(),
        }
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn step(&mut self) {
        self.sample_clock += SAMPLES_PER_CYCLE;

        if self.sample_clock >= SAMPLE_PERIOD {
            self.sample_clock -= SAMPLE_PERIOD;
            self.audio_queue.push_back((0.0, 0.0));
        }
    }
}

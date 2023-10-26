use super::constants::RATE;
use tracing::trace;

pub struct NoiseGenerator {
    divider: Option<u32>,
    counter: Option<u32>,
    level: i32,
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self {
            divider: None,
            counter: None,
            level: -0x4000,
        }
    }

    pub fn level(&self) -> i32 {
        self.level
    }

    pub fn set_rate(&mut self, value: u8) {
        self.divider = RATE[value as usize];
        self.counter = self.divider;
        trace!("Noise Divider: {:?}", self.divider);
    }

    pub fn step(&mut self) {
        self.counter = self.counter.map(|counter| counter - 1);

        if !self.counter.is_some_and(|counter| counter == 0) {
            return;
        }

        self.counter = self.divider;

        let feedback = (self.level & 1) ^ ((self.level >> 1) & 1);
        self.level = ((self.level >> 1) & 0x3fff) | (feedback << 14);

        // Sign extend 15-bit result
        self.level = (self.level << 17) >> 17;

        trace!("Noise Level: {}", self.level);
    }
}

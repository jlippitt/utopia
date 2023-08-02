#[rustfmt::skip]
const PERIODS: [u32; 32] = [
     10, 254,  20,   2,  40,   4,  80,   6,
    160,   8,  60,  10,  14,  12,  26,  14,
     12,  16,  24,  18,  48,  20,  96,  22,
    192,  24,  72,  26,  16,  28,  32,  30,
];

pub struct LengthCounter {
    enabled: bool,
    halted: bool,
    counter: u32,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            enabled: false,
            halted: false,
            counter: 0,
        }
    }

    pub fn counter(&self) -> u32 {
        self.counter
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.counter = 0;
        }
    }

    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }

    pub fn load(&mut self, value: u8) {
        if self.enabled {
            self.counter = PERIODS[value as usize];
        }
    }

    pub fn step(&mut self) {
        if self.counter > 0 && !self.halted {
            self.counter -= 1;
        }
    }
}

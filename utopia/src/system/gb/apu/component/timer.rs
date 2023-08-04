pub struct Timer {
    counter: u32,
    period: u32,
}

impl Timer {
    pub const MAX_VALUE: u32 = 2047;

    pub fn new() -> Self {
        Self {
            counter: 0,
            period: 0,
        }
    }

    pub fn reset(&mut self) {
        self.counter = self.period;
    }

    pub fn period(&self) -> u32 {
        self.period
    }

    pub fn set_period(&mut self, value: u32) {
        self.period = value;
    }

    pub fn set_period_low(&mut self, value: u8) {
        self.period = (self.period & 0xff00) | (value as u32);
    }

    pub fn set_period_high(&mut self, value: u8) {
        self.period = (self.period & 0xff) | ((value as u32) << 8);
    }

    pub fn step(&mut self) -> bool {
        self.counter += 1;

        if self.counter > Self::MAX_VALUE {
            self.counter = self.period;
            return true;
        }

        false
    }
}

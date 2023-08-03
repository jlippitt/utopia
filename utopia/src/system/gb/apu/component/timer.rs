const COUNTER_MAX: u32 = 2048;

pub struct Timer {
    counter: u32,
    period: u32,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            period: 0,
        }
    }

    pub fn reset(&mut self) {
        self.counter = self.period;
    }

    pub fn set_period_low(&mut self, value: u8) {
        self.period = (self.period & 0xff00) | (value as u32);
    }

    pub fn set_period_high(&mut self, value: u8) {
        self.period = (self.period & 0xff) | ((value as u32) << 8);
    }

    pub fn step(&mut self) -> bool {
        self.counter += 1;

        if self.counter == COUNTER_MAX {
            self.counter = self.period;
            return true;
        }

        false
    }
}

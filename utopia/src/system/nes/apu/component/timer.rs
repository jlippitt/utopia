pub struct Timer {
    counter: u32,
    period: u32,
    shift: u32,
}

impl Timer {
    pub fn new(period: u32, shift: u32) -> Self {
        Self {
            counter: (1 << shift) - 1,
            period,
            shift,
        }
    }

    pub fn period(&self) -> u32 {
        self.period
    }

    pub fn set_period(&mut self, period: u32) {
        self.period = period;
    }

    pub fn set_period_low(&mut self, value: u8) {
        self.period = (self.period & 0xff00) | (value as u32);
    }

    pub fn set_period_high(&mut self, value: u8) {
        self.period = (self.period & 0xff) | ((value as u32) << 8);
    }

    pub fn step(&mut self) -> bool {
        if self.counter == 0 {
            self.counter = ((self.period + 1) << self.shift) - 1;
            true
        } else {
            self.counter -= 1;
            false
        }
    }
}

pub struct Timer {
    counter: u32,
    period: u32,
    frequency: u32,
}

impl Timer {
    pub const MAX_FREQUENCY: u32 = 2047;
    pub const MAX_PERIOD: u32 = Self::MAX_FREQUENCY + 1;

    pub fn new(period: u32) -> Self {
        Self {
            counter: period,
            period,
            frequency: 0,
        }
    }

    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    pub fn reset(&mut self) {
        self.counter = self.period;
    }

    pub fn set_period(&mut self, period: u32) {
        assert!(period != 0);
        self.period = period;
    }

    pub fn set_frequency(&mut self, frequency: u32) {
        self.frequency = frequency;
        self.period = Self::MAX_PERIOD - self.frequency;
    }

    pub fn set_frequency_low(&mut self, value: u8) {
        self.frequency = (self.frequency & 0xff00) | (value as u32);
        self.period = Self::MAX_PERIOD - self.frequency;
    }

    pub fn set_frequency_high(&mut self, value: u8) {
        self.frequency = (self.frequency & 0xff) | ((value as u32) << 8);
        self.period = Self::MAX_PERIOD - self.frequency;
    }

    pub fn step(&mut self) -> bool {
        self.counter -= 1;

        if self.counter == 0 {
            self.counter = self.period;
            return true;
        }

        false
    }
}

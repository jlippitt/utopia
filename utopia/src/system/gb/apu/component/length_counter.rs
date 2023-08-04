pub struct LengthCounter {
    counter: u32,
    period: u32,
    enabled: bool,
}

impl LengthCounter {
    const MAX_VALUE: u32 = 63;

    pub fn new() -> Self {
        Self {
            counter: 0,
            period: 0,
            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        if self.counter == Self::MAX_VALUE {
            self.counter = 0;
        }
    }

    pub fn set_period(&mut self, value: u32) {
        self.period = value;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn step(&mut self) -> bool {
        if !self.enabled || self.counter > Self::MAX_VALUE {
            return false;
        }

        self.counter += 1;

        self.counter > Self::MAX_VALUE
    }
}

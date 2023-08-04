pub struct LengthCounter {
    counter: u32,
    max_value: u32,
    period: u32,
    enabled: bool,
}

impl LengthCounter {
    pub fn new(max_value: u32) -> Self {
        Self {
            counter: 0,
            max_value,
            period: 0,
            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        if self.counter == self.max_value {
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
        if !self.enabled || self.counter > self.max_value {
            return false;
        }

        self.counter += 1;

        self.counter > self.max_value
    }
}

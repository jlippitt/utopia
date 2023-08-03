const COUNTER_MAX: u32 = 64;

pub struct LengthCounter {
    counter: u32,
    period: u32,
    enabled: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            period: 0,
            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        if self.counter == COUNTER_MAX {
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
        if !self.enabled || self.counter >= COUNTER_MAX {
            return false;
        }

        self.counter += 1;

        self.counter == COUNTER_MAX
    }
}

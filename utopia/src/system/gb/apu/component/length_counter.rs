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
            period: max_value,
            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        if self.counter == 0 {
            self.counter = self.max_value;
        }
    }

    pub fn set_period(&mut self, value: u32) {
        self.period = self.max_value - value;

        if self.enabled {
            self.counter = self.period;
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if self.enabled {
            self.counter = self.period;
        }
    }

    pub fn step(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        println!("{}", self.counter);

        if self.counter != 0 {
            self.counter -= 1;
        }

        self.counter == 0
    }
}

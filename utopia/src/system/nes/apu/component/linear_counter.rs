pub struct LinearCounter {
    counter: u32,
    period: u32,
    reload: bool,
    control: bool,
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            period: 0,
            reload: false,
            control: false,
        }
    }

    pub fn counter(&self) -> u32 {
        self.counter
    }

    pub fn set_control(&mut self, value: u8) {
        self.control = (value & 0x80) != 0;
        self.period = value as u32 & 0x7f;
    }

    pub fn reset(&mut self) {
        self.reload = true;
    }

    pub fn step(&mut self) {
        if self.reload {
            self.counter = self.period;

            if !self.control {
                self.reload = false;
            }
        } else if self.counter > 0 {
            self.counter -= 1;
        }
    }
}

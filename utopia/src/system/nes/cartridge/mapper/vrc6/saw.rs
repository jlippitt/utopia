const ACCUMULATOR_STEPS: u8 = 14;

pub struct Saw {
    freq_counter: u16,
    freq_period: u16,
    freq_shift: u8,
    enabled: bool,
    accum_step: u8,
    accum_value: u8,
    accum_rate: u8,
}

impl Saw {
    pub fn new() -> Self {
        Self {
            freq_counter: 0,
            freq_period: 0,
            freq_shift: 0,
            enabled: false,
            accum_step: 0,
            accum_value: 0,
            accum_rate: 0,
        }
    }

    pub fn output(&self) -> u8 {
        if self.enabled {
            self.accum_value >> 3
        } else {
            0
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.accum_rate = value & 0x3f;
    }

    pub fn set_freq_period_low(&mut self, value: u8) {
        self.freq_period = (self.freq_period & 0xff00) | value as u16;
    }

    pub fn set_freq_period_high(&mut self, value: u8) {
        self.freq_period = (self.freq_period & 0xff) | ((value as u16 & 0x0f) << 8);
        self.enabled = (value & 0x80) != 0;

        if !self.enabled {
            self.accum_step = 0;
            self.accum_value = 0;
        }
    }

    pub fn set_freq_shift(&mut self, freq_shift: u8) {
        self.freq_shift = freq_shift;
    }

    pub fn step(&mut self) {
        if self.freq_counter != 0 {
            self.freq_counter -= 1;
            return;
        }

        self.freq_counter = self.freq_period >> self.freq_shift;

        if !self.enabled {
            return;
        }

        self.accum_step += 1;

        if (self.accum_step & 1) == 0 {
            self.accum_value = self.accum_value.wrapping_add(self.accum_rate);
        }

        if self.accum_step == ACCUMULATOR_STEPS {
            self.accum_step = 0;
            self.accum_value = 0;
        }
    }
}

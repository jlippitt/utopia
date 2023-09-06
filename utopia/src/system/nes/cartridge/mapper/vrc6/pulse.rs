pub struct Pulse {
    freq_counter: u16,
    freq_period: u16,
    freq_shift: u8,
    enabled: bool,
    duty_counter: u8,
    duty_threshold: u8,
    volume: u8,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            freq_counter: 0,
            freq_period: 0,
            freq_shift: 0,
            enabled: false,
            duty_counter: 0,
            duty_threshold: 0,
            volume: 0,
        }
    }

    pub fn output(&self) -> u8 {
        if self.enabled && self.duty_counter <= self.duty_threshold {
            self.volume
        } else {
            0
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.duty_threshold = if (value & 0x80) == 0 {
            (value >> 4) & 7
        } else {
            15
        };

        self.volume = value & 15;
    }

    pub fn set_freq_period_low(&mut self, value: u8) {
        self.freq_period = (self.freq_period & 0xff00) | value as u16;
    }

    pub fn set_freq_period_high(&mut self, value: u8) {
        self.freq_period = (self.freq_period & 0xff) | ((value as u16 & 0x0f) << 8);
        self.enabled = (value & 0x80) != 0;

        if !self.enabled {
            self.duty_counter = 15;
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

        if self.enabled {
            self.duty_counter = self.duty_counter.wrapping_sub(1) & 15;
        }
    }
}

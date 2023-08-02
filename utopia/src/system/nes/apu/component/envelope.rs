pub struct Envelope {
    constant_volume: bool,
    decay: u8,
    divider: u8,
    period: u8,
    start_flag: bool,
    loop_flag: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            constant_volume: false,
            decay: 0,
            divider: 0,
            period: 0,
            start_flag: false,
            loop_flag: false,
        }
    }

    pub fn volume(&self) -> u8 {
        if self.constant_volume {
            self.period
        } else {
            self.decay
        }
    }

    pub fn reset(&mut self) {
        self.start_flag = true;
    }

    pub fn set_control(&mut self, value: u8) {
        self.loop_flag = (value & 0x20) != 0;
        self.constant_volume = (value & 0x10) != 0;
        self.period = value & 15;
    }

    pub fn step(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay = 15;
            self.divider = self.period;
            return;
        }

        if self.divider > 0 {
            self.divider -= 1;
            return;
        }

        self.divider = self.period;

        if self.decay > 0 {
            self.decay -= 1;
        } else if self.loop_flag {
            self.decay = 15;
        }
    }
}

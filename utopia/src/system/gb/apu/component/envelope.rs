pub struct Envelope {
    volume: u8,
    initial: u8,
    counter: u32,
    period: u32,
    increment: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            volume: 0,
            initial: 0,
            counter: 0,
            period: 0,
            increment: false,
        }
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn reset(&mut self) {
        self.volume = self.initial;
        self.counter = self.period;
    }

    pub fn set_control(&mut self, value: u8) {
        self.initial = value >> 4;
        self.increment = (value & 0x08) != 0;
        self.period = value as u32 & 0x07;
    }

    pub fn step(&mut self) {
        if self.counter == 0 {
            return;
        }

        self.counter -= 1;

        if self.counter != 0 {
            return;
        }

        if self.increment && self.volume < 15 {
            self.volume += 1;
            self.counter = self.period;
        } else if !self.increment && self.volume > 0 {
            self.volume -= 1;
            self.counter = self.period;
        }
    }
}

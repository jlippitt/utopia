use super::component::{LengthCounter, Timer};

const VOLUME_SHIFT: [u32; 4] = [4, 0, 1, 2];

pub struct Wave {
    power: bool,
    enabled: bool,
    timer: Timer,
    sample: u8,
    read_index: usize,
    volume_shift: u32,
    length_counter: LengthCounter,
    sample_ram: [u8; 16],
    read_value: [u8; 5],
}

impl Wave {
    pub fn new() -> Self {
        Self {
            power: false,
            enabled: false,
            timer: Timer::new(Timer::MAX_PERIOD),
            sample: 0,
            read_index: 0,
            volume_shift: VOLUME_SHIFT[0],
            length_counter: LengthCounter::new(256),
            sample_ram: [0; 16],
            read_value: [0xff; 5],
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn output(&self) -> u8 {
        if self.enabled {
            self.sample >> self.volume_shift
        } else {
            0
        }
    }

    pub fn read_register(&mut self, address: u8) -> u8 {
        self.read_value[address as usize]
    }

    pub fn write_register(&mut self, address: u8, value: u8) {
        match address {
            0 => {
                self.power = (value & 0x80) != 0;
                self.read_value[0] = 0x7f | (value & 0x80);

                if !self.power {
                    self.enabled = false;
                }
            }
            1 => self.length_counter.set_period(value as u32),
            2 => {
                self.volume_shift = VOLUME_SHIFT[(value as usize >> 5) & 3];
                self.read_value[2] = 0x9f | (value & 0x60);
            }
            3 => self.timer.set_frequency_low(value),
            4 => {
                self.timer.set_frequency_high(value & 0x07);
                self.length_counter.set_enabled((value & 0x40) != 0);
                self.read_value[4] = 0xbf | (value & 0x40);

                if (value & 0x80) != 0 {
                    self.enabled = self.power;
                    self.timer.reset();
                    self.read_index = 0;
                    self.length_counter.reset();
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn write_ram(&mut self, index: usize, value: u8) {
        self.sample_ram[index] = value;
    }

    pub fn step(&mut self) {
        if !(self.timer.step() | self.timer.step()) {
            return;
        }

        self.read_index = (self.read_index + 1) & 31;

        let byte = self.sample_ram[self.read_index >> 1];

        self.sample = if (self.read_index & 1) != 0 {
            byte & 15
        } else {
            byte >> 4
        };
    }

    pub fn on_divider_clock(&mut self, divider: u64) {
        if (divider & 1) == 0 && self.length_counter.step() {
            self.enabled = false;
        }
    }
}

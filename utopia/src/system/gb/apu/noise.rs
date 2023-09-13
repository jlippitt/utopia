use super::component::{Envelope, LengthCounter, Timer};

const DIVIDER: [u32; 8] = [2, 4, 8, 12, 16, 20, 24, 28];

pub struct Noise {
    power: bool,
    enabled: bool,
    timer: Timer,
    shift: u16,
    mode: bool,
    length_counter: LengthCounter,
    envelope: Envelope,
    read_value: [u8; 5],
}

impl Noise {
    pub fn new() -> Self {
        Self {
            power: false,
            enabled: false,
            timer: Timer::new(DIVIDER[0]),
            shift: 0x7fff,
            mode: false,
            length_counter: LengthCounter::new(64),
            envelope: Envelope::new(),
            read_value: [0xff; 5],
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn output(&self) -> u8 {
        if (self.shift & 1) == 0 && self.enabled {
            self.envelope.volume()
        } else {
            0
        }
    }

    pub fn read(&mut self, address: u8) -> u8 {
        self.read_value[address as usize]
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0 => (),
            1 => self.length_counter.set_period(value as u32 & 0x3f),
            2 => {
                self.envelope.set_control(value);
                self.power = (value & 0xf8) != 0;
                self.read_value[2] = value;

                if !self.power {
                    self.enabled = false;
                }
            }
            3 => {
                let shift = value >> 4;
                let divider = DIVIDER[value as usize & 7];
                self.timer.set_period(divider << shift);
                self.mode = (value & 0x08) != 0;
                self.read_value[3] = value;
            }
            4 => {
                self.length_counter.set_enabled((value & 0x40) != 0);
                self.read_value[4] = 0xbf | (value & 0x40);

                if (value & 0x80) != 0 {
                    self.enabled = self.power;
                    self.timer.reset();
                    self.shift = 0x7fff;
                    self.length_counter.reset();
                    self.envelope.reset();
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self) {
        if !self.timer.step() {
            return;
        }

        let feedback = (self.shift ^ (self.shift >> 1)) & 1;
        self.shift = (feedback << 14) | (self.shift >> 1);

        if self.mode {
            self.shift = (self.shift & !0x40) | (feedback << 6);
        }
    }

    pub fn on_divider_clock(&mut self, divider: u64) {
        if (divider & 1) == 0 && self.length_counter.step() {
            self.enabled = false;
        }

        if (divider & 7) == 7 {
            self.envelope.step();
        }
    }
}

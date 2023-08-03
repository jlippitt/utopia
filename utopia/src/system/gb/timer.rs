use super::apu::Apu;
use super::interrupt::{Interrupt, InterruptType};
use tracing::debug;

const PERIOD_MASKS: [u64; 4] = [1023, 15, 63, 255];
const APU_DIVIDER_MASK: u64 = 8191;

struct Control {
    enable: bool,
    period_mask: u64,
    raw: u8,
}

pub struct Timer {
    divider: u64,
    counter: u8,
    modulo: u8,
    control: Control,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider: 0,
            counter: 0,
            modulo: 0,
            control: Control {
                enable: false,
                period_mask: PERIOD_MASKS[0],
                raw: 0,
            },
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        match address {
            4 => (self.divider >> 8) as u8,
            5 => self.counter,
            6 => self.modulo,
            7 => 0xf8 | self.control.raw,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, apu: &mut Apu, address: u8, value: u8) {
        match address {
            4 => {
                if (self.divider & APU_DIVIDER_MASK) != 0 {
                    apu.clock_divider();
                }

                self.divider = 0;
                debug!("Timer Divider Reset");
            }
            5 => {
                self.counter = value;
                debug!("Timer Counter: {}", self.counter);
            }
            6 => {
                self.modulo = value;
                debug!("Timer Modulo: {}", self.modulo);
            }
            7 => {
                self.control.raw = value;
                self.control.enable = (value & 0x04) != 0;
                self.control.period_mask = PERIOD_MASKS[value as usize & 0x03];
                debug!("Timer Enable: {}", self.control.enable);
                debug!("Timer Period Mask: {}", self.control.period_mask);
            }
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt, apu: &mut Apu, cycles: u64) {
        self.divider += cycles;

        if (self.divider & APU_DIVIDER_MASK) == 0 {
            apu.clock_divider();
        }

        if !self.control.enable {
            return;
        }

        if (self.divider & self.control.period_mask) < cycles {
            self.counter = self.counter.wrapping_add(1);

            if self.counter == 0 {
                self.counter = self.modulo;
                interrupt.raise(InterruptType::Timer);
            }

            debug!("Timer Counter: {}", self.counter);
        }
    }
}

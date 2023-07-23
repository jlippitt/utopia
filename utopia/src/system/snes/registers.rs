use super::VBLANK_LINE;
use crate::core::wdc65c816::{Interrupt, INT_NMI};
use tracing::debug;

pub const TIMER_IRQ: Interrupt = 0x04;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum IrqMode {
    None,
    H,
    V,
    HV,
}

pub struct Registers {
    irq_mode: IrqMode,
    irq_x: u16,
    irq_y: u16,
    multiplicand: u8,
    dividend: u16,
    quotient: u16,
    remainder: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            irq_mode: IrqMode::None,
            irq_x: 0x01ff,
            irq_y: 0x01ff,
            multiplicand: 0xff,
            dividend: 0xffff,
            quotient: 0xffff,
            remainder: 0xffff,
        }
    }

    pub fn irq_cycle(&self, line: u16) -> Option<u64> {
        match self.irq_mode {
            IrqMode::None => None,
            IrqMode::V => {
                if line == self.irq_y {
                    Some(0)
                } else {
                    None
                }
            }
            IrqMode::H => Some((self.irq_x as u64) << 2),
            IrqMode::HV => {
                if line == self.irq_y {
                    Some((self.irq_x as u64) << 2)
                } else {
                    None
                }
            }
        }
    }

    pub fn multiply(&mut self, value: u8) {
        // TODO: Simulate hardware delay
        self.remainder = (self.multiplicand as u16) * (value as u16);

        debug!(
            "Multiplication (Unsigned): {} * {} = {}",
            self.multiplicand, value, self.remainder
        );
    }

    pub fn divide(&mut self, value: u8) {
        // TODO: Simulate hardware delay
        if value != 0 {
            self.quotient = self.dividend / (value as u16);
            self.remainder = self.dividend % (value as u16);
        } else {
            self.quotient = 0xffff;
            self.remainder = self.dividend;
        }

        debug!(
            "Division (Unsigned): {} / {} = {} ({})",
            self.dividend, value, self.quotient, self.remainder
        );
    }
}

impl super::Hardware {
    pub(super) fn read_register(&mut self, address: u8, prev_value: u8) -> u8 {
        match address {
            0x10 => {
                let mut value = (prev_value & 0x70) | 0x02;

                if self.clock.nmi_occurred() {
                    self.interrupt &= !INT_NMI;
                    value |= 0x80;
                }

                self.clock.set_nmi_occurred(&mut self.interrupt, false);

                value
            }
            0x11 => {
                let mut value = prev_value & 0x7f;

                if (self.interrupt & TIMER_IRQ) != 0 {
                    self.interrupt &= !TIMER_IRQ;
                    value |= 0x80;
                }

                value
            }
            0x12 => {
                let line = self.clock.line();
                let dot = self.clock.dot();

                let mut value = prev_value & 0x3e;

                if line >= VBLANK_LINE {
                    // VBlank
                    value |= 0x80;

                    if line < (VBLANK_LINE + 3) {
                        // Auto Joypad Read
                        value |= 0x01;
                    }
                }

                if dot >= 274 || dot == 0 {
                    // HBlank
                    value |= 0x40;
                }

                value
            }
            0x14 => self.regs.quotient as u8,
            0x15 => (self.regs.quotient >> 8) as u8,
            0x16 => self.regs.remainder as u8,
            0x17 => (self.regs.remainder >> 8) as u8,
            0x18..=0x1f => 0, // TODO: Auto joypad read
            _ => todo!("Register read {:02X}", address),
        }
    }

    pub(super) fn write_register(&mut self, address: u8, value: u8) {
        match address {
            0x00 => {
                // TODO: Auto-joypad read

                self.clock
                    .set_nmi_active(&mut self.interrupt, (value & 0x80) != 0);

                self.regs.irq_mode = match value & 0x30 {
                    0x00 => IrqMode::None,
                    0x10 => IrqMode::H,
                    0x20 => IrqMode::V,
                    0x30 => IrqMode::HV,
                    _ => unreachable!(),
                };

                debug!("IRQ Mode: {:?}", self.regs.irq_mode);
            }
            0x02 => {
                self.regs.multiplicand = value;
                debug!("Multiplicand: {}", self.regs.multiplicand);
            }
            0x03 => self.regs.multiply(value),
            0x04 => {
                self.regs.dividend = (self.regs.dividend & 0xff00) | (value as u16);
                debug!("Dividend: {}", self.regs.dividend);
            }
            0x05 => {
                self.regs.dividend = (self.regs.dividend & 0xff) | ((value as u16) << 8);
                debug!("Dividend: {}", self.regs.dividend);
            }
            0x06 => self.regs.divide(value),
            0x07 => {
                self.regs.irq_x = (self.regs.irq_x & 0xff00) | (value as u16);
                debug!("IRQ X: {}", self.regs.irq_x);
            }
            0x08 => {
                self.regs.irq_x = (self.regs.irq_x & 0xff) | ((value as u16 & 0x01) << 8);
                debug!("IRQ X: {}", self.regs.irq_x);
            }
            0x09 => {
                self.regs.irq_y = (self.regs.irq_y & 0xff00) | (value as u16);
                debug!("IRQ Y: {}", self.regs.irq_y);
            }
            0x0a => {
                self.regs.irq_y = (self.regs.irq_y & 0xff) | ((value as u16 & 0x01) << 8);
                debug!("IRQ Y: {}", self.regs.irq_y);
            }
            0x0b => self.dma.set_dma_enabled(value),
            _ => (),
        }
    }
}

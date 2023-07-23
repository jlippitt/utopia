use super::clock::TIMER_IRQ;
use super::VBLANK_LINE;
use tracing::debug;

pub struct Registers {
    multiplicand: u8,
    dividend: u16,
    quotient: u16,
    remainder: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            multiplicand: 0xff,
            dividend: 0xffff,
            quotient: 0xffff,
            remainder: 0xffff,
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

                self.clock
                    .set_irq_mode(&mut self.interrupt, (value >> 4) & 0x03);
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
            0x07 => self.clock.set_irq_x_low(value),
            0x08 => self.clock.set_irq_x_high(value),
            0x09 => self.clock.set_irq_y_low(value),
            0x0a => self.clock.set_irq_y_high(value),
            0x0b => self.dma.set_dma_enabled(value),
            0x0c => self.dma.set_hdma_enabled(value),
            _ => (),
        }
    }
}

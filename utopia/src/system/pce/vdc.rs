use super::interrupt::{Interrupt, InterruptType};
use bitflags::bitflags;
use tracing::{debug, warn};

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct VdcInterrupt: u8 {
        const SPRITE_COLLISION = 0x01;
        const SPRITE_OVERFLOW = 0x02;
        const SCANLINE = 0x04;
        const VBLANK = 0x08;
    }
}

pub struct Vdc {
    reg_index: u8,
    interrupt_enable: VdcInterrupt,
    interrupt_active: VdcInterrupt,
    scanline_match: u16,
    display_width: u16,
    display_height: u16,
    interrupt: Interrupt,
}

impl Vdc {
    pub const DEFAULT_WIDTH: u16 = 256;
    pub const DEFAULT_HEIGHT: u16 = 224;

    pub fn new(interrupt: Interrupt) -> Self {
        Self {
            reg_index: 0,
            interrupt_enable: VdcInterrupt::empty(),
            interrupt_active: VdcInterrupt::empty(),
            scanline_match: 0,
            display_width: Self::DEFAULT_WIDTH,
            display_height: Self::DEFAULT_HEIGHT,
            interrupt,
        }
    }

    pub fn display_width(&self) -> u16 {
        self.display_width
    }

    pub fn display_height(&self) -> u16 {
        self.display_height
    }

    pub fn scanline_match(&self) -> u16 {
        self.scanline_match
    }

    pub fn read(&mut self, address: u16, _prev_value: u8) -> u8 {
        match address & 3 {
            0 => {
                let mut status = self.interrupt_active.bits();

                // VBlank bit is annoyingly moved to bit 5
                status = (status & 0x07) | ((status & 0x08) << 2);

                // TODO: DMA status

                self.interrupt_active = VdcInterrupt::empty();
                debug!("VDC Interrupts Cleared");
                self.interrupt.clear(InterruptType::Irq1);

                status
            }
            _ => unimplemented!("VDC Read: {:04X}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => self.reg_index = value & 0x1f,
            2 => self.write_register(value, false),
            3 => self.write_register(value, true),
            _ => warn!("Unimplemented VDC Write: {:04X} <= {:02X}", address, value),
        }
    }

    pub fn raise_interrupt(&mut self, int_type: VdcInterrupt) {
        if !self.interrupt_enable.contains(int_type) {
            return;
        }

        self.interrupt_active |= int_type;
        debug!("VDC Interrupt Raised: {:?}", int_type);
        self.interrupt.raise(InterruptType::Irq1);
    }

    fn write_register(&mut self, value: u8, msb: bool) {
        match self.reg_index {
            0x05 => {
                // TODO: Other settings
                if !msb {
                    self.interrupt_enable = VdcInterrupt::from_bits_retain(value & 0x0f);
                    debug!("VDC Interrupt Enable: {:?}", self.interrupt_enable);
                }
            }
            0x06 => {
                self.scanline_match = if msb {
                    (self.scanline_match & 0xff) | ((value as u16 & 0x03) << 8)
                } else {
                    (self.scanline_match & 0xff00) | value as u16
                };
                debug!("VDC Scanline Match: {}", self.scanline_match,);
            }
            0x0b => {
                if !msb {
                    self.display_width = ((value as u16 & 0x3f) + 1) << 3;
                    debug!("VDC Display Width: {}", self.display_width)
                }
            }
            0x0d => {
                let last_display_line = if msb {
                    ((self.display_height - 1) & 0xff) | ((value as u16 & 0x01) << 8)
                } else {
                    ((self.display_height - 1) & 0xff00) | (value as u16)
                };

                self.display_height = last_display_line + 1;

                debug!("VDC Display Height: {}", self.display_height)
            }
            _ => warn!(
                "Unimplemented VDC Register Write: {:02X} <= {:04X}",
                self.reg_index, value
            ),
        }
    }
}

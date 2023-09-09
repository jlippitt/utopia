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
    write_buffer: u8,
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
            write_buffer: 0,
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

    pub fn read(&self, address: u16, _prev_value: u8) -> u8 {
        unimplemented!("VDC Read: {:04X}", address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => self.reg_index = value & 0x1f,
            2 => self.write_buffer = value,
            3 => self.write_register(value),
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

    pub fn clear_interrupt(&mut self, int_type: VdcInterrupt) {
        if !self.interrupt_active.contains(int_type) {
            return;
        }

        self.interrupt_active &= int_type;
        debug!("VDC Interrupt Cleared: {:?}", int_type);

        if self.interrupt_active & self.interrupt_enable == VdcInterrupt::empty() {
            self.interrupt.clear(InterruptType::Irq1);
        }
    }

    fn write_register(&mut self, high_byte: u8) {
        let value = ((high_byte as u16) << 8) | self.write_buffer as u16;

        match self.reg_index {
            0x05 => {
                // TODO: Other settings
                self.interrupt_enable = VdcInterrupt::from_bits_retain(value as u8 & 0x0f);
                debug!("VDC Interrupt Enable: {:?}", self.interrupt_enable);
            }
            0x06 => {
                self.scanline_match = value & 0x03ff;
                debug!("VDC Scanline Match: {}", self.scanline_match,);
            }
            0x0b => {
                self.display_width = ((value & 0x3f) + 1) << 3;
                debug!("VDC Display Width: {}", self.display_width)
            }
            0x0d => {
                self.display_height = (value & 0x01ff) + 1;
                debug!("VDC Display Height: {}", self.display_height)
            }
            _ => warn!(
                "Unimplemented VDC Register Write: {:02X} <= {:04X}",
                self.reg_index, value
            ),
        }
    }
}

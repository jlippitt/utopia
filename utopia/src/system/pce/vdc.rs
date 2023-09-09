use bitflags::bitflags;
use tracing::{debug, warn};

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct InterruptEnable: u8 {
        const SPRITE_COLLISION = 0x01;
        const SPRITE_OVERFLOW = 0x02;
        const SCANLINE = 0x04;
        const VBLANK = 0x08;
    }
}

pub struct Vdc {
    reg_index: u8,
    write_buffer: u8,
    interrupt_enable: InterruptEnable,
}

impl Vdc {
    pub fn new() -> Self {
        Self {
            reg_index: 0,
            write_buffer: 0,
            interrupt_enable: InterruptEnable::empty(),
        }
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

    fn write_register(&mut self, high_byte: u8) {
        let value = ((high_byte as u16) << 8) | self.write_buffer as u16;

        match self.reg_index {
            0x05 => {
                // TODO: Other settings
                self.interrupt_enable = InterruptEnable::from_bits_retain(value as u8 & 0x0f);
                debug!("VDC Interrupt Enable: {:?}", self.interrupt_enable);
            }
            _ => warn!(
                "Unimplemented VDC Register Write: {:02X} <= {:04X}",
                self.reg_index, value
            ),
        }
    }
}

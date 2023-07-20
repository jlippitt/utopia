use crate::util::MirrorVec;
use tracing::debug;

const CGRAM_SIZE: usize = 128;

pub struct Cgram {
    data: MirrorVec<u16>,
    address: u8,
    high_byte: bool,
    buffer: u8,
}

impl Cgram {
    pub fn new() -> Self {
        Self {
            data: MirrorVec::new(CGRAM_SIZE),
            address: 0,
            high_byte: false,
            buffer: 0,
        }
    }

    pub fn color(&mut self, index: usize) -> u16 {
        self.data[index]
    }

    pub fn set_address(&mut self, value: u8) {
        self.address = value;
        self.high_byte = false;
        debug!("CGRAM Address: {:02X}", self.address);
    }

    pub fn write(&mut self, value: u8) {
        if self.high_byte {
            let value = ((value as u16 & 0x7f) << 8) | (self.buffer as u16);
            self.data[self.address as usize] = value;
            debug!("CGRAM Write: {:02X} <= {:04X}", self.address, value);
            self.address = self.address.wrapping_add(1);
        } else {
            self.buffer = value;
        }

        self.high_byte = !self.high_byte;
    }
}

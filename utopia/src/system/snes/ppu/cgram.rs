use crate::util::MirrorVec;
use tracing::trace;

const CGRAM_SIZE: usize = 256;

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
        trace!("CGRAM Address: {:02X}", self.address);
    }

    pub fn read(&mut self) -> u8 {
        let address = self.address as usize;

        let value = if self.high_byte {
            let value = (self.data[address] >> 8) as u8;
            self.address = self.address.wrapping_add(1);
            value
        } else {
            self.data[address] as u8
        };

        trace!(
            "CGRAM Read: {:02X}.{} => {:02X}",
            address,
            self.high_byte as u32,
            value
        );

        self.high_byte = !self.high_byte;

        value
    }

    pub fn write(&mut self, value: u8) {
        if self.high_byte {
            let word_value = ((value as u16 & 0x7f) << 8) | (self.buffer as u16);
            self.data[self.address as usize] = word_value;
            trace!("CGRAM Write: {:02X} <= {:04X}", self.address, word_value);
            self.address = self.address.wrapping_add(1);
        } else {
            self.buffer = value;
        }

        self.high_byte = !self.high_byte;
    }
}

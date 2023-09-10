use crate::util::MirrorVec;
use tracing::debug;

const VRAM_WORD_SIZE: usize = 32768;

pub struct Vram {
    write_address: u16,
    increment_amount: u16,
    write_buffer: u8,
    data: MirrorVec<u16>,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            write_address: 0,
            increment_amount: 1,
            write_buffer: 0,
            data: MirrorVec::new(VRAM_WORD_SIZE),
        }
    }

    pub fn data(&self, address: usize) -> u16 {
        self.data[address]
    }

    pub fn set_write_address(&mut self, msb: bool, value: u8) {
        self.write_address = if msb {
            (self.write_address & 0xff) | ((value as u16) << 8)
        } else {
            (self.write_address & 0xff00) | value as u16
        };

        debug!("VRAM Write Address: {:04X}", self.write_address);
    }

    pub fn set_increment_amount(&mut self, value: u16) {
        self.increment_amount = value;
        debug!("VRAM Increment Amount: {}", self.increment_amount);
    }

    pub fn write(&mut self, msb: bool, value: u8) {
        if !msb {
            self.write_buffer = value;
            return;
        }

        let word_value = ((value as u16) << 8) | (self.write_buffer as u16);

        self.data[self.write_address as usize] = word_value;

        debug!(
            "VRAM Write: {:04X} <= {:04X}",
            self.write_address, word_value
        );

        self.write_address = self.write_address.wrapping_add(self.increment_amount);
    }
}

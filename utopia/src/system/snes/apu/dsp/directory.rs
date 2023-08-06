use crate::util::MirrorVec;
use tracing::debug;

pub struct Directory {
    base_address: u16,
}

impl Directory {
    pub fn new() -> Self {
        Self { base_address: 0 }
    }

    pub fn set_base_address(&mut self, value: u8) {
        self.base_address = (value as u16) << 8;
        debug!("DIR Base Address: {:04X}", self.base_address);
    }

    pub fn start_address(&self, ram: &MirrorVec<u8>, source: u8) -> u16 {
        let low = self.byte(ram, source, 0);
        let high = self.byte(ram, source, 1);
        let address = u16::from_le_bytes([low, high]);
        debug!("Start Address: {:04X}", address);
        address
    }

    fn byte(&self, ram: &MirrorVec<u8>, source: u8, byte: usize) -> u8 {
        let address = self.base_address as usize + ((source as usize) << 2) + byte;
        let value = ram[address];
        debug!("DIR Read: {:04X} => {:02X}", address, value);
        value
    }
}

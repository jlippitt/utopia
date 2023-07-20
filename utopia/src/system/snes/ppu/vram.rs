use crate::util::MirrorVec;
use tracing::debug;

const VRAM_SIZE: usize = 32768;

pub struct Vram {
    data: MirrorVec<u16>,
    address: u16,
    increment_high: bool,
    increment_amount: u16,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            data: MirrorVec::new(VRAM_SIZE),
            address: 0,
            increment_high: false,
            increment_amount: 1,
        }
    }

    pub fn data(&self, address: u16) -> u16 {
        self.data[address as usize]
    }

    pub fn set_control(&mut self, value: u8) {
        if (value & 0x0c) != 0 {
            todo!("VRAM address remapping");
        }

        self.increment_amount = match value & 0x03 {
            0 => 1,
            1 => 32,
            _ => 128,
        };

        self.increment_high = (value & 0x80) != 0;

        debug!("VRAM Increment Amount: {}", self.increment_amount);
        debug!("VRAM Increment High: {}", self.increment_high);
    }

    pub fn set_address_low(&mut self, value: u8) {
        self.address = (self.address & 0xff00) | (value as u16);
        debug!("VRAM Address: {:04X}", self.address);
    }

    pub fn set_address_high(&mut self, value: u8) {
        self.address = (self.address & 0xff) | ((value as u16) << 8);
        debug!("VRAM Address: {:04X}", self.address);
    }

    pub fn write_low(&mut self, value: u8) {
        let address = self.address as usize;
        self.data[address] = (self.data[address] & 0xff00) | (value as u16);

        debug!(
            "VRAM Write (Low): {:04X} <= {:02X} ({:04X})",
            address, value, self.data[address]
        );

        if !self.increment_high {
            self.address = self.address.wrapping_add(self.increment_amount);
        }
    }

    pub fn write_high(&mut self, value: u8) {
        let address = self.address as usize;
        self.data[address] = (self.data[address] & 0xff) | ((value as u16) << 8);

        debug!(
            "VRAM Write (High): {:04X} <= {:02X} ({:04X})",
            address, value, self.data[address]
        );

        if self.increment_high {
            self.address = self.address.wrapping_add(self.increment_amount);
        }
    }
}

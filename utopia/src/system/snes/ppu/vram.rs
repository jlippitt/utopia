use crate::util::MirrorVec;
use tracing::debug;

const VRAM_SIZE: usize = 32768;

const PLANE_0_MASK: u16 = 0x5555;

pub struct Vram {
    data: MirrorVec<u16>,
    chr_cache: MirrorVec<u16>,
    address: u16,
    increment_high: bool,
    increment_amount: u16,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            data: MirrorVec::new(VRAM_SIZE),
            chr_cache: MirrorVec::new(VRAM_SIZE),
            address: 0,
            increment_high: false,
            increment_amount: 1,
        }
    }

    pub fn data(&self, address: u16) -> u16 {
        self.data[address as usize]
    }

    pub fn chr4(&self, address: u16) -> u16 {
        self.chr_cache[address as usize]
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

        self.update_chr_cache(address, 0, value as u16);

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

        self.update_chr_cache(address, 1, value as u16);

        if self.increment_high {
            self.address = self.address.wrapping_add(self.increment_amount);
        }
    }

    fn update_chr_cache(&mut self, address: usize, plane: u8, value: u16) {
        let index = ((address << 12) | (address >> 3)) & 0x7fff;

        let chr_value = ((value & 0x01) << 14)
            | ((value & 0x02) << 11)
            | ((value & 0x04) << 8)
            | ((value & 0x08) << 5)
            | ((value & 0x10) << 2)
            | ((value & 0x20) >> 1)
            | ((value & 0x40) >> 4)
            | ((value & 0x80) >> 7);

        self.chr_cache[index] =
            self.chr_cache[index] & !(PLANE_0_MASK << plane) | (chr_value << plane);
    }
}

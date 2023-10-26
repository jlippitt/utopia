use crate::util::MirrorVec;
use tracing::trace;

const VRAM_SIZE: usize = 32768;

const PLANE_0_MASK: u16 = 0x5555;

pub struct Vram {
    data: MirrorVec<u16>,
    chr_cache: MirrorVec<u16>,
    remap_mode: u8,
    address: u16,
    increment_high: bool,
    increment_amount: u16,
    read_buffer: u16,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            data: MirrorVec::new(VRAM_SIZE),
            chr_cache: MirrorVec::new(VRAM_SIZE),
            remap_mode: 0,
            address: 0,
            increment_high: false,
            increment_amount: 1,
            read_buffer: 0,
        }
    }

    pub fn data(&self, address: usize) -> u16 {
        self.data[address]
    }

    pub fn chr4(&self, index: usize) -> u64 {
        self.chr_cache[index] as u64
    }

    pub fn chr16(&self, index: usize) -> u64 {
        let plane0 = self.chr_cache[index] as u64;
        let plane1 = self.chr_cache[index | 8] as u64;
        (plane1 << 16) | plane0
    }

    pub fn chr256(&self, index: usize) -> u64 {
        let plane0 = self.chr_cache[index] as u64;
        let plane1 = self.chr_cache[index | 8] as u64;
        let plane2 = self.chr_cache[index | 16] as u64;
        let plane3 = self.chr_cache[index | 24] as u64;
        (plane3 << 48) | (plane2 << 32) | (plane1 << 16) | plane0
    }

    pub fn set_control(&mut self, value: u8) {
        self.remap_mode = (value & 0x0c) >> 2;

        self.increment_amount = match value & 0x03 {
            0 => 1,
            1 => 32,
            _ => 128,
        };

        self.increment_high = (value & 0x80) != 0;

        trace!("VRAM Remap Mode: {}", self.remap_mode);
        trace!("VRAM Increment Amount: {}", self.increment_amount);
        trace!("VRAM Increment High: {}", self.increment_high);
    }

    pub fn set_address_low(&mut self, value: u8) {
        self.address = (self.address & 0xff00) | (value as u16);
        trace!("VRAM Address: {:04X}", self.address);
        self.prefetch();
    }

    pub fn set_address_high(&mut self, value: u8) {
        self.address = (self.address & 0xff) | ((value as u16) << 8);
        trace!("VRAM Address: {:04X}", self.address);
        self.prefetch();
    }

    pub fn read_low(&mut self) -> u8 {
        let value = self.read_buffer as u8;

        if !self.increment_high {
            self.prefetch();
            self.address = self.address.wrapping_add(self.increment_amount);
        }

        value
    }

    pub fn read_high(&mut self) -> u8 {
        let value = (self.read_buffer >> 8) as u8;

        if self.increment_high {
            self.prefetch();
            self.address = self.address.wrapping_add(self.increment_amount);
        }

        value
    }

    pub fn write_low(&mut self, value: u8) {
        let address = self.remap_address();
        self.data[address] = (self.data[address] & 0xff00) | (value as u16);

        trace!(
            "VRAM Write (Low): {:04X} <= {:02X} ({:04X})",
            address,
            value,
            self.data[address]
        );

        self.update_chr_cache(address, 0, value as u16);

        if !self.increment_high {
            self.address = self.address.wrapping_add(self.increment_amount);
        }
    }

    pub fn write_high(&mut self, value: u8) {
        let address = self.remap_address();
        self.data[address] = (self.data[address] & 0xff) | ((value as u16) << 8);

        trace!(
            "VRAM Write (High): {:04X} <= {:02X} ({:04X})",
            address,
            value,
            self.data[address]
        );

        self.update_chr_cache(address, 1, value as u16);

        if self.increment_high {
            self.address = self.address.wrapping_add(self.increment_amount);
        }
    }

    fn remap_address(&self) -> usize {
        let address = self.address as usize;

        match self.remap_mode {
            0 => address & 0x7fff,
            1 => (address & 0x7f00) | ((address << 3) & 0x00f8) | ((address >> 5) & 0x07),
            2 => (address & 0x7e00) | ((address << 3) & 0x01f8) | ((address >> 6) & 0x07),
            3 => (address & 0x7c00) | ((address << 3) & 0x03f8) | ((address >> 7) & 0x07),
            _ => unreachable!(),
        }
    }

    fn prefetch(&mut self) {
        let address = self.remap_address();
        self.read_buffer = self.data[address];
        trace!("VRAM Read: {:04X} => {:04X}", address, self.read_buffer);
    }

    fn update_chr_cache(&mut self, address: usize, plane: u8, value: u16) {
        let chr_value = ((value & 0x01) << 14)
            | ((value & 0x02) << 11)
            | ((value & 0x04) << 8)
            | ((value & 0x08) << 5)
            | ((value & 0x10) << 2)
            | ((value & 0x20) >> 1)
            | ((value & 0x40) >> 4)
            | ((value & 0x80) >> 7);

        self.chr_cache[address] =
            (self.chr_cache[address] & !(PLANE_0_MASK << plane)) | (chr_value << plane);

        trace!(
            "CHR Cache Write (Plane {}): {:04X} <= {:02X} ({:04X})",
            plane,
            address,
            value,
            self.chr_cache[address]
        );
    }
}

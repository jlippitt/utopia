use crate::util::MirrorVec;
use std::ops::{Index, IndexMut};
use tracing::{debug, warn};

const WRAM_SIZE: usize = 131072;

pub struct Wram {
    data: MirrorVec<u8>,
    address: u32,
}

impl Wram {
    pub fn new() -> Self {
        Self {
            data: MirrorVec::new(WRAM_SIZE),
            address: 0,
        }
    }

    pub fn read_register(&mut self, address: u8, prev_value: u8) -> u8 {
        match address & 0x3f {
            0x00 => {
                let value = self.data[self.address as usize];
                debug!("WRAM Read: {:04X} => {:02X}", self.address, value);
                self.address = self.address.wrapping_add(1);
                value
            }
            _ => {
                warn!("Unmapped WRAM register read: {:02X}", address);
                prev_value
            }
        }
    }

    pub fn write_register(&mut self, address: u8, value: u8) {
        match address & 0x3f {
            0x00 => {
                self.data[self.address as usize] = value;
                debug!("WRAM Write: {:04X} <= {:02X}", self.address, value);
                self.address = self.address.wrapping_add(1);
            }
            0x01 => {
                self.address = (self.address & 0xffff_ff00) | (value as u32);
                debug!("WRAM Address: {:04X}", self.address);
            }
            0x02 => {
                self.address = (self.address & 0xffff_00ff) | ((value as u32) << 8);
                debug!("WRAM Address: {:04X}", self.address);
            }
            0x03 => {
                self.address = (self.address & 0xff00_ffff) | ((value as u32) << 16);
                debug!("WRAM Address: {:04X}", self.address);
            }
            _ => warn!(
                "Unmapped WRAM register write: {:02X} <= {:02X}",
                address, value
            ),
        }
    }
}

impl Index<usize> for Wram {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.data[index]
    }
}

impl IndexMut<usize> for Wram {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.data[index]
    }
}

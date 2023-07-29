use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

const MI_VERSION: u32 = 0x0202_0102;

pub struct MipsInterface {
    mi_mode: u16,
}

impl MipsInterface {
    pub fn new() -> Self {
        Self { mi_mode: 0 }
    }
}

impl DataReader for MipsInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address & 0x0f {
            0x00 => self.mi_mode as u32 & 0x03ff,
            0x04 => MI_VERSION,
            _ => unimplemented!("MIPS Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for MipsInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                self.mi_mode = (value as u16) & 0x3fff;
                debug!("MI_MODE: {:04X}", value);
            }
            0x0c => {
                // TODO: MI_MASK
            }
            _ => unimplemented!("MIPS Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

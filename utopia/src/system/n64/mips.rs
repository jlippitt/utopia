use super::interrupt::RcpInterrupt;
use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

const MI_VERSION: u32 = 0x0202_0102;

pub struct MipsInterface {
    mode: u16,
    mask: u8,
    interrupt: RcpInterrupt,
}

impl MipsInterface {
    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            mode: 0,
            mask: 0,
            interrupt,
        }
    }
}

impl DataReader for MipsInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address & 0x0f {
            0x00 => self.mode as u32 & 0x03ff,
            0x04 => MI_VERSION,
            0x0c => self.mask as u32,
            _ => unimplemented!("MIPS Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for MipsInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                self.mode = (value as u16) & 0x3fff;
                debug!("MI_MODE: {:04X}", self.mode);
            }
            0x0c => {
                for bit in 0..6 {
                    match (value >> (bit << 1)) & 3 {
                        0 => (),
                        1 => self.mask &= !(1 << bit),
                        2 => self.mask |= 1 << bit,
                        _ => panic!("Invalid MI_MASK write: {:012b}", value),
                    }
                }

                self.interrupt.set_mask(self.mask);
            }
            _ => unimplemented!("MIPS Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

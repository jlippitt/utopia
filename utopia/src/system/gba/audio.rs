use crate::util::facade::{DataReader, DataWriter};
use tracing::{debug, warn};

pub struct Audio {
    bias: u16,
}

impl Audio {
    pub fn new() -> Self {
        Self { bias: 0x0200 }
    }
}

impl DataReader for Audio {
    type Address = u32;
    type Value = u16;

    fn read(&self, address: u32) -> u16 {
        match address & 0xff {
            0x88 => self.bias,
            address => panic!("Unmapped Audio Read: {:02X}", address),
        }
    }
}

impl DataWriter for Audio {
    fn write(&mut self, address: u32, value: u16) {
        match address & 0xff {
            0x88 => {
                self.bias = value & 0xc3fe;
                debug!("Audio Bias: {:04X}", self.bias);
            }
            address => warn!("Unmapped Audio Write: {:02X} <= {:04X}", address, value),
        }
    }
}

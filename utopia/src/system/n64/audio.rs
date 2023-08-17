use crate::util::facade::{DataReader, DataWriter};
use tracing::warn;

pub struct AudioInterface {}

impl AudioInterface {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataReader for AudioInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address & 0x0f {
            0x0c => {
                // AI_STATUS
                // TODO
                0x0110_0000
            }
            _ => unimplemented!("Audio Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for AudioInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x0c => {
                // AI_STATUS
                // TODO: Acknowledge AI interrupt
            }
            _ => warn!("Audio Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

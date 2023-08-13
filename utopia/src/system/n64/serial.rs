use crate::util::facade::{DataReader, DataWriter};

pub struct SerialInterface {}

impl SerialInterface {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataReader for SerialInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            0x18 => {
                // SI_STATUS
                // TODO
                0
            }
            _ => unimplemented!("Serial Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for SerialInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x18 => {
                // SI_STATUS
                // There are no writable bits here
            }
            _ => unimplemented!("Serial Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

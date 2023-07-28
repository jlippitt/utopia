use crate::util::facade::{DataReader, DataWriter};

pub struct Interface {
    //
}

pub struct Rdram {
    interface: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            interface: Interface {},
        }
    }

    pub fn interface(&self) -> &Interface {
        &self.interface
    }

    pub fn interface_mut(&mut self) -> &mut Interface {
        &mut self.interface
    }
}

impl DataReader for Interface {
    type Address = u32;
    type Value = u32;

    fn read(&self, _address: u32) -> u32 {
        // TODO
        0
    }
}

impl DataWriter for Interface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x04 => {
                // RI_CONFIG
                // Ignore
            }
            _ => unimplemented!("RDRAM Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

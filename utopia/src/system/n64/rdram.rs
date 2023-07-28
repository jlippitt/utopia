use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

pub struct Interface {
    ri_select: u32,
}

pub struct Rdram {
    interface: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            interface: Interface { ri_select: 0 },
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

    fn read(&self, address: u32) -> u32 {
        match address {
            0x0c => self.ri_select,
            _ => unimplemented!("RDRAM Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for Interface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x04 => (), // RI_CONFIG: Ignore
            0x08 => (), // RI_CURRENT_LOAD: Ignore
            0x0c => {
                self.ri_select = value & 0xff;
                debug!("RI_SELECT: {:08X}", value);
            }
            _ => unimplemented!("RDRAM Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

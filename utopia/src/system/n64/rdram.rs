use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

pub struct Interface {
    ri_mode: u8,
    ri_select: u8,
}

pub struct Rdram {
    interface: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            interface: Interface {
                ri_mode: 0,
                ri_select: 0,
            },
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
            0x00 => self.ri_mode as u32,
            0x0c => self.ri_select as u32,
            _ => unimplemented!("RDRAM Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for Interface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                self.ri_mode = (value as u8) & 0x0f;
                debug!("RI_MODE: {:02X}", value);
            }
            0x04 => (), // RI_CONFIG: Ignore
            0x08 => (), // RI_CURRENT_LOAD: Ignore
            0x0c => {
                self.ri_select = value as u8;
                debug!("RI_SELECT: {:02X}", value);
            }
            _ => unimplemented!("RDRAM Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

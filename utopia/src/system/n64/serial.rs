use crate::util::facade::{DataReader, DataWriter};

pub struct SerialBus {
    interface: Interface,
}

impl SerialBus {
    pub fn new() -> Self {
        Self {
            interface: Interface::new(),
        }
    }

    pub fn interface(&self) -> &Interface {
        &self.interface
    }

    pub fn interface_mut(&mut self) -> &mut Interface {
        &mut self.interface
    }
}

impl DataReader for SerialBus {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            _ => unimplemented!("Serial Bus Read: {:08X}", address),
        }
    }
}

impl DataWriter for SerialBus {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            _ => unimplemented!("Serial Bus Write: {:08X} <= {:08X}", address, value),
        }
    }
}

pub struct Interface {}

impl Interface {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataReader for Interface {
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

impl DataWriter for Interface {
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

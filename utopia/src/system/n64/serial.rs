use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};

pub struct SerialBus {
    interface: Interface,
    pif_ram: [u8; 64],
}

impl SerialBus {
    pub fn new() -> Self {
        Self {
            interface: Interface::new(),
            pif_ram: [0; 64],
        }
    }

    pub fn interface(&self) -> &Interface {
        &self.interface
    }

    pub fn interface_mut(&mut self) -> &mut Interface {
        &mut self.interface
    }

    pub fn read<T: Value>(&self, address: u32) -> T {
        match address {
            0x0000_007c0..=0x0000_007ff => self.pif_ram.as_slice().read_be(address as usize & 63),
            _ => unimplemented!("Serial Bus Read: {:08X}", address),
        }
    }

    pub fn write<T: Value>(&mut self, address: u32, value: T) {
        match address {
            0x0000_007c0..=0x0000_007ff => self
                .pif_ram
                .as_mut_slice()
                .write_be(address as usize & 63, value),
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

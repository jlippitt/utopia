use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use tracing::debug;

const RDRAM_SIZE: usize = 1024 * 1024 * 8;

pub struct Registers {
    device_id: u16,
    mode: u32,
}

pub struct Interface {
    ri_mode: u8,
    ri_select: u8,
    ri_refresh: u32,
}

pub struct Rdram {
    data: Vec<u8>,
    registers: Registers,
    interface: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            data: vec![0; RDRAM_SIZE],
            registers: Registers {
                device_id: 0,
                mode: 0,
            },
            interface: Interface {
                ri_mode: 0,
                ri_select: 0,
                ri_refresh: 0,
            },
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn read_data<T: Value>(&self, address: u32) -> T {
        self.data.read_be(address as usize)
    }

    pub fn write_data<T: Value>(&mut self, address: u32, value: T) {
        self.data.write_be(address as usize, value);
    }

    pub fn read_register<T: Value>(&self, address: u32) -> T {
        if address < 0x0008_0000 {
            self.registers.read_be(address & 0x0007_ffff)
        } else {
            // Broadcast area is write-only
            T::default()
        }
    }

    pub fn write_register<T: Value>(&mut self, address: u32, value: T) {
        // TODO: How does broadcasting work?
        self.registers.write_be(address & 0x0007_ffff, value);
    }

    pub fn read_interface<T: Value>(&self, address: u32) -> T {
        self.interface.read_be(address)
    }

    pub fn write_interface<T: Value>(&mut self, address: u32, value: T) {
        self.interface.write_be(address, value);
    }
}

impl DataReader for Registers {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        // Mask the address (at least for now) to mirror registers
        let address = address & 0x0000_03ff;

        let value = match address {
            0x0c => self.mode,
            _ => unimplemented!("RDRAM Register Read: {:08X}", address),
        };

        // RDRAM registers are little-endian
        value.swap_bytes()
    }
}

impl DataWriter for Registers {
    fn write(&mut self, address: u32, value: u32) {
        // Mask the address (at least for now) to mirror registers
        let address = address & 0x0000_03ff;

        // RDRAM registers are little-endian
        let value = value.swap_bytes();

        match address {
            0x04 => {
                let device_id = ((value & 0x0000_00fc) >> 2)
                    | ((value & 0x00ff_8000) >> 9)
                    | ((value & 0x8000_0000) >> 16);

                self.device_id = device_id as u16;
                debug!("RDRAM Device ID: {:04X}", self.device_id);
            }
            0x08 => (), // Delay: Ignore
            0x0c => {
                self.mode = value & 0xc0c0c0ef;

                // Some fields are inverted when read back
                self.mode ^= 0xc0c0c040;

                debug!("RDRAM Mode: {:08X}", self.mode);
            }
            0x14 => (), // RefRow: Ignore
            _ => unimplemented!("RDRAM Register Write: {:08X} <= {:08X}", address, value),
        }
    }
}

impl DataReader for Interface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            0x00 => self.ri_mode as u32,
            0x0c => self.ri_select as u32,
            0x10 => self.ri_refresh,
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
            0x10 => {
                self.ri_refresh = value & 0x001f_ffff;
                debug!("RI_REFRESH: {:08X}", value);
            }
            _ => unimplemented!("RDRAM Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

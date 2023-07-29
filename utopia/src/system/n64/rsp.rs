use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use tracing::debug;

pub const DMEM_SIZE: usize = 4096;

const IMEM_SIZE: usize = 4096;
const RAM_SIZE: usize = DMEM_SIZE + IMEM_SIZE;

pub struct Rsp {
    hw: Hardware,
}

impl Rsp {
    pub fn new<T: Into<Vec<u8>>>(dmem: T) -> Self {
        let mut ram = dmem.into();

        assert!(ram.len() == DMEM_SIZE);

        ram.resize(RAM_SIZE, 0);

        Self {
            hw: Hardware::new(ram),
        }
    }

    pub fn read<T: Value>(&self, address: u32) -> T {
        if address < 0x0004_0000 {
            self.hw.ram.read_be(address as usize & (RAM_SIZE - 1))
        } else {
            self.hw.read_be(address)
        }
    }

    pub fn write<T: Value>(&mut self, address: u32, value: T) {
        if address < 0x0004_0000 {
            self.hw
                .ram
                .write_be(address as usize & (RAM_SIZE - 1), value);
        } else {
            self.hw.write_be(address, value);
        }
    }
}

struct Hardware {
    ram: Vec<u8>,
    pc: u32,
}

impl Hardware {
    fn new(ram: Vec<u8>) -> Self {
        Self { ram, pc: 0 }
    }
}

impl DataReader for Hardware {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: Self::Address) -> Self::Value {
        match address {
            0x0008_0000 => self.pc,
            _ => unimplemented!("RSP Register Read: {:08X}", address),
        }
    }
}

impl DataWriter for Hardware {
    fn write(&mut self, address: Self::Address, value: Self::Value) {
        match address {
            0x0008_0000 => {
                self.pc = value & 0x0ffc;
                debug!("RSP Program Counter: {:08X}", self.pc);
            }
            _ => unimplemented!("RSP Register Write: {:08X} <= {:08X}", address, value),
        }
    }
}

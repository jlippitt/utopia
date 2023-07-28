use crate::util::facade::{ReadFacade, Value};

pub const DMEM_SIZE: usize = 4096;

const IMEM_SIZE: usize = 4096;
const RAM_SIZE: usize = DMEM_SIZE + IMEM_SIZE;

pub struct Rsp {
    ram: Vec<u8>,
}

impl Rsp {
    pub fn new<T: Into<Vec<u8>>>(dmem: T) -> Self {
        let mut ram = dmem.into();

        assert!(ram.len() == DMEM_SIZE);

        ram.resize(RAM_SIZE, 0);

        Self { ram }
    }

    pub fn read<T: Value>(&self, address: u32) -> T {
        if address & 0x1_0000 < 0x4_0000 {
            self.ram.read_be(address as usize & (RAM_SIZE - 1))
        } else {
            todo!("RSP Registers");
        }
    }
}

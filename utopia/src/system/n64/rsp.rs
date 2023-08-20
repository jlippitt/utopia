use super::dma::Dma;
use crate::core::mips::{Bus, Core, Interrupt};
use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use crate::util::MirrorVec;
use cp0::Cp0;
use tracing::debug;

mod cp0;

pub const DMEM_SIZE: usize = 4096;

const IMEM_SIZE: usize = 4096;

pub struct Rsp {
    core: Core<Hardware>,
}

impl Rsp {
    pub fn new<T: Into<Vec<u8>>>(dmem: T) -> Self {
        let dmem = dmem.into();

        assert!(dmem.len() == DMEM_SIZE);

        Self {
            core: Core::new(Hardware::new(dmem), Default::default()),
        }
    }

    pub fn read_ram<T: Value>(&self, address: u32) -> T {
        // TODO: Mirroring
        let address = address as usize;

        if address < DMEM_SIZE {
            self.core.bus().dmem.read_be(address)
        } else {
            self.core.bus().imem.read_be(address)
        }
    }

    pub fn write_ram<T: Value>(&mut self, address: u32, value: T) {
        // TODO: Mirroring
        let address = address as usize;

        if address < DMEM_SIZE {
            self.core.bus_mut().dmem.write_be(address, value);
        } else {
            self.core.bus_mut().imem.write_be(address, value);
        }
    }

    pub fn dma_requested(&self) -> Dma {
        self.core.cp0().dma_requested()
    }

    pub fn finish_dma(&mut self) {
        self.core.cp0_mut().finish_dma()
    }
}

impl DataReader for Rsp {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: Self::Address) -> Self::Value {
        match address {
            0x0004_0000..=0x0004_001F => self.core.cp0().get((address as usize >> 2) & 7),
            0x0008_0000 => self.core.pc(),
            _ => unimplemented!("RSP Register Read: {:08X}", address),
        }
    }
}

impl DataWriter for Rsp {
    fn write(&mut self, address: Self::Address, value: Self::Value) {
        match address {
            0x0004_0000..=0x0004_001F => {
                let start = self.core.cp0_mut().set((address as usize >> 2) & 7, value);

                if start {
                    debug!("***BEGIN RSP***");

                    loop {
                        self.core.step();
                    }
                }
            }
            0x0008_0000 => {
                self.core.set_pc(value & 0x0ffc);
                debug!("RSP Program Counter: {:04X}", value & 0x0ffc);
            }
            _ => unimplemented!("RSP Register Write: {:08X} <= {:08X}", address, value),
        }
    }
}

struct Hardware {
    dmem: MirrorVec<u8>,
    imem: MirrorVec<u8>,
}

impl Hardware {
    fn new(dmem: Vec<u8>) -> Self {
        Self {
            dmem: dmem.into(),
            imem: MirrorVec::new(IMEM_SIZE),
        }
    }
}

impl Bus for Hardware {
    type Cp0 = Cp0;
    type Cp2 = ();

    const CP1: bool = false;
    const MUL_DIV: bool = false;
    const INSTR_64: bool = false;
    const PC_MASK: u32 = 0xfff;

    fn read_opcode<T: Value>(&mut self, address: u32) -> T {
        self.imem.read_be(address as usize)
    }

    fn read<T: Value>(&mut self, address: u32) -> T {
        self.dmem.read_be(address as usize)
    }

    fn write<T: Value>(&mut self, address: u32, value: T) {
        self.dmem.write_be(address as usize, value);
    }

    fn step(&mut self) {
        // TODO
    }

    fn poll(&self) -> Interrupt {
        // No interrupts in RSP
        0
    }
}

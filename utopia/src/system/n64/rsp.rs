use super::dma::{Dma, DmaRequest};
use crate::core::mips::{Bus, Core, Interrupt};
use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use tracing::debug;

pub const DMEM_SIZE: usize = 4096;

const IMEM_SIZE: usize = 4096;

pub struct Rsp {
    dma_requested: Dma,
    dma_spaddr: u32,
    dma_ramaddr: u32,
    core: Core<Hardware>,
}

impl Rsp {
    pub fn new<T: Into<Vec<u8>>>(dmem: T) -> Self {
        let dmem = dmem.into();

        assert!(dmem.len() == DMEM_SIZE);

        Self {
            dma_requested: Dma::None,
            dma_spaddr: 0,
            dma_ramaddr: 0,
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
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = Dma::None;
        // TODO: Interrupt?
    }
}

impl DataReader for Rsp {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: Self::Address) -> Self::Value {
        match address {
            0x0004_0010 => {
                // SP_STATUS
                // TODO
                0x01
            }
            0x0008_0000 => self.core.pc(),
            _ => unimplemented!("RSP Register Read: {:08X}", address),
        }
    }
}

impl DataWriter for Rsp {
    fn write(&mut self, address: Self::Address, value: Self::Value) {
        match address {
            0x0004_0000 => {
                self.dma_spaddr = value & 0x1ff8;
                debug!("SP_DMA_SPADDR: {:08X}", self.dma_spaddr);
            }
            0x0004_0004 => {
                self.dma_ramaddr = value & 0x00ff_fff8;
                debug!("SP_DMA_RAMADDR: {:08X}", self.dma_ramaddr);
            }
            0x0004_0008 => {
                self.dma_requested = Dma::Read(DmaRequest {
                    src_addr: self.dma_spaddr,
                    dst_addr: self.dma_ramaddr,
                    len: value & 0xff8f_fff8,
                })
            }
            0x0004_0010 => {
                // SP_STATUS
                // TODO
                if (value & 1) != 0 {
                    todo!("RSP");
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
    dmem: Vec<u8>,
    imem: Vec<u8>,
}

impl Hardware {
    fn new(dmem: Vec<u8>) -> Self {
        Self {
            dmem,
            imem: vec![0; IMEM_SIZE],
        }
    }
}

impl Bus for Hardware {
    const CP0: bool = false;
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

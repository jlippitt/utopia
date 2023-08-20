use super::dma::{Dma, DmaRequest};
use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use tracing::debug;

pub const DMEM_SIZE: usize = 4096;

const IMEM_SIZE: usize = 4096;
const RAM_SIZE: usize = DMEM_SIZE + IMEM_SIZE;

pub struct Rsp {
    dma_requested: Dma,
    dma_spaddr: u32,
    dma_ramaddr: u32,
    hw: Hardware,
}

impl Rsp {
    pub fn new<T: Into<Vec<u8>>>(dmem: T) -> Self {
        let mut ram = dmem.into();

        assert!(ram.len() == DMEM_SIZE);

        ram.resize(RAM_SIZE, 0);

        Self {
            dma_requested: Dma::None,
            dma_spaddr: 0,
            dma_ramaddr: 0,
            hw: Hardware::new(ram),
        }
    }

    pub fn read_ram<T: Value>(&self, address: u32) -> T {
        self.hw.ram.read_be(address as usize)
    }

    pub fn write_ram<T: Value>(&mut self, address: u32, value: T) {
        self.hw.ram.write_be(address as usize, value);
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
            0x0008_0000 => self.hw.pc,
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
                self.hw.pc = value & 0x0ffc;
                debug!("RSP Program Counter: {:08X}", self.hw.pc);
            }
            _ => unimplemented!("RSP Register Write: {:08X} <= {:08X}", address, value),
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

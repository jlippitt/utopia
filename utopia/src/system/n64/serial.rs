use super::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use tracing::debug;

pub struct SerialBus {
    interface: Interface,
    pif: [u8; 2048],
}

impl SerialBus {
    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            interface: Interface::new(interrupt),
            pif: [0; 2048],
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
            0x0000_00000..=0x0000_007ff => self.pif.as_slice().read_be(address as usize),
            _ => unimplemented!("Serial Bus Read: {:08X}", address),
        }
    }

    pub fn write<T: Value>(&mut self, address: u32, value: T) {
        match address {
            0x0000_007c0..=0x0000_007ff => {
                // Only the top 64 bytes (PIF RAM) are writable
                self.pif.as_mut_slice().write_be(address as usize, value)
            }
            _ => unimplemented!("Serial Bus Write: {:08X} <= {:08X}", address, value),
        }
    }

    pub fn dma_requested(&self) -> PifDma {
        self.interface.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.interface.dma_requested = PifDma::None;
        self.interface.interrupt.raise(RcpIntType::SI);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PifDmaRequest {
    pub dram_addr: u32,
    pub pif_addr: u32,
    pub len: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum PifDma {
    None,
    Read(PifDmaRequest),
    Write(PifDmaRequest),
}

pub struct Interface {
    dma_requested: PifDma,
    dram_addr: u32,
    interrupt: RcpInterrupt,
}

impl Interface {
    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            dma_requested: PifDma::None,
            dram_addr: 0,
            interrupt,
        }
    }
}

impl DataReader for Interface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            0x18 => {
                // SI_STATUS
                match self.dma_requested {
                    PifDma::None => 0,
                    _ => 0x1000,
                }
            }
            _ => unimplemented!("Serial Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for Interface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                self.dram_addr = value & 0x00ff_ffff;
                debug!("SI_DRAM_ADDR: {:08X}", self.dram_addr);
            }
            0x04 => {
                self.dma_requested = PifDma::Read(PifDmaRequest {
                    dram_addr: self.dram_addr,
                    pif_addr: value & 0x07fc,
                    len: 64,
                });
            }
            0x10 => {
                self.dma_requested = PifDma::Write(PifDmaRequest {
                    dram_addr: self.dram_addr,
                    pif_addr: value & 0x07fc,
                    len: 64,
                });
            }
            0x18 => {
                // SI_STATUS
                self.interrupt.clear(RcpIntType::SI);
            }
            _ => unimplemented!("Serial Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

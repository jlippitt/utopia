use super::dma::{Dma, DmaRequest};
use super::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::facade::{DataReader, DataWriter};
use crate::JoypadState;
use pif::Pif;
use tracing::debug;

mod pif;

pub struct SerialInterface {
    dma_requested: Dma,
    dram_addr: u32,
    interrupt: RcpInterrupt,
    pif: Pif,
}

impl SerialInterface {
    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            dma_requested: Dma::None,
            dram_addr: 0,
            interrupt,
            pif: Pif::new(),
        }
    }

    pub fn pif(&self) -> &Pif {
        &self.pif
    }

    pub fn pif_mut(&mut self) -> &mut Pif {
        &mut self.pif
    }

    pub fn dma_requested(&self) -> Dma {
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = Dma::None;
        self.interrupt.raise(RcpIntType::SI);
        self.pif.upload();
    }

    pub fn update_joypad(&mut self, state: &JoypadState) {
        self.pif.update_joypad(state);
    }
}

impl DataReader for SerialInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            0x18 => {
                // SI_STATUS
                match self.dma_requested {
                    Dma::None => 0,
                    _ => 0x1000,
                }
            }
            _ => unimplemented!("Serial Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for SerialInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                self.dram_addr = value & 0x00ff_ffff;
                debug!("SI_DRAM_ADDR: {:08X}", self.dram_addr);
            }
            0x04 => {
                self.pif.process();

                self.dma_requested = Dma::Read(DmaRequest {
                    src_addr: self.dram_addr,
                    dst_addr: value & 0x07fc,
                    len: 64,
                });
            }
            0x10 => {
                self.dma_requested = Dma::Write(DmaRequest {
                    src_addr: self.dram_addr,
                    dst_addr: value & 0x07fc,
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

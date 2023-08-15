use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

#[derive(Copy, Clone, Debug)]
pub struct DmaRequest {
    pub dram_address: u32,
    pub cart_address: u32,
    pub len: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum Dma {
    None,
    //Read(DmaRequest),
    Write(DmaRequest),
}

pub struct PeripheralInterface {
    dma_requested: Dma,
    dram_address: u32,
    cart_address: u32,
}

impl PeripheralInterface {
    pub fn new() -> Self {
        Self {
            dma_requested: Dma::None,
            dram_address: 0,
            cart_address: 0,
        }
    }

    pub fn dma_requested(&self) -> Dma {
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = Dma::None;
    }
}

impl DataReader for PeripheralInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: Self::Address) -> Self::Value {
        match address {
            0x00 => self.dram_address,
            0x04 => self.cart_address,
            0x0c => 0x7f,
            0x10 => {
                // TODO: Other PI_STATUS bits
                match self.dma_requested {
                    Dma::None => 0,
                    _ => 1,
                }
            }
            0x14 => {
                // PI_BSD_DOM1_LAT
                // TODO: Set from ROM header
                64
            }
            0x18 => {
                // PI_BSD_DOM1_PWD
                // TODO: Set from ROM header
                18
            }
            0x1c => {
                // PI_BSD_DOM1_PGS
                // TODO: Set from ROM header
                7
            }
            0x20 => {
                // PI_BSD_DOM1_RLS
                // TODO: Set from ROM header
                3
            }
            0x24 | 0x28 | 0x2c | 0x30 => {
                // PI_BSD_DOM2
                // TODO
                0
            }
            _ => unimplemented!("Peripheral Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for PeripheralInterface {
    fn write(&mut self, address: Self::Address, value: Self::Value) {
        match address {
            0x00 => {
                self.dram_address = value & 0x00ff_fffe;
                debug!("PI DRAM Address: {:08X}", self.dram_address);
            }
            0x04 => {
                self.cart_address = value & 0xffff_fffe;
                debug!("PI CART Address: {:08X}", self.cart_address);
            }
            0x0c => {
                self.dma_requested = Dma::Write(DmaRequest {
                    dram_address: self.dram_address,
                    cart_address: self.cart_address,
                    len: value & 0x00ff_ffff,
                });
            }
            0x10 => {
                // PI_STATUS
                // TODO: Clear interrupt
                // TODO: Reset DMA controller
            }
            _ => unimplemented!(
                "Peripheral Interface Write: {:08X} <= {:08X}",
                address,
                value
            ),
        }
    }
}

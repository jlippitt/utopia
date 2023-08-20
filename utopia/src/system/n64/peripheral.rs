use super::dma::Dma;
use super::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

struct BsdDom {
    lat: u8,
    pwd: u8,
    pgs: u8,
    rls: u8,
}

pub struct PeripheralInterface {
    dma_requested: Option<Dma>,
    dram_address: u32,
    cart_address: u32,
    bsd_dom: [BsdDom; 2],
    interrupt: RcpInterrupt,
}

impl PeripheralInterface {
    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            dma_requested: None,
            dram_address: 0,
            cart_address: 0,
            bsd_dom: [
                // TODO: Set from ROM header
                BsdDom {
                    lat: 64,
                    pwd: 18,
                    pgs: 7,
                    rls: 3,
                },
                BsdDom {
                    lat: 0,
                    pwd: 0,
                    pgs: 0,
                    rls: 0,
                },
            ],
            interrupt,
        }
    }

    pub fn dma_requested(&self) -> Option<Dma> {
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = None;
        self.interrupt.raise(RcpIntType::PI);
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
                let mut value = 0;

                value |= if self.interrupt.has(RcpIntType::PI) {
                    0x08
                } else {
                    0
                };

                value |= if self.dma_requested.is_some() {
                    0x01
                } else {
                    0
                };

                value
            }
            0x14 => self.bsd_dom[0].lat as u32,
            0x18 => self.bsd_dom[0].pwd as u32,
            0x1c => self.bsd_dom[0].pgs as u32,
            0x20 => self.bsd_dom[0].rls as u32,
            0x24 => self.bsd_dom[1].lat as u32,
            0x28 => self.bsd_dom[1].pwd as u32,
            0x2c => self.bsd_dom[1].pgs as u32,
            0x30 => self.bsd_dom[1].rls as u32,
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
                self.dma_requested = Some(Dma {
                    src_addr: self.cart_address,
                    dst_addr: self.dram_address,
                    len: value & 0x00ff_ffff,
                    reverse: false,
                });
            }
            0x10 => {
                // PI_STATUS
                if (value & 2) != 0 {
                    self.interrupt.clear(RcpIntType::PI);
                }

                // TODO: Reset DMA controller
            }
            0x14 => {
                self.bsd_dom[0].lat = value as u8;
                debug!("PI BSD DOM1 LAT: {}", self.bsd_dom[0].lat);
            }
            0x18 => {
                self.bsd_dom[0].pwd = value as u8;
                debug!("PI BSD DOM1 PWD: {}", self.bsd_dom[0].pwd);
            }
            0x1c => {
                self.bsd_dom[0].pgs = value as u8 & 15;
                debug!("PI BSD DOM1 PGS: {}", self.bsd_dom[0].pgs);
            }
            0x20 => {
                self.bsd_dom[0].rls = value as u8 & 3;
                debug!("PI BSD DOM1 RLS: {}", self.bsd_dom[0].rls);
            }
            0x24 => {
                self.bsd_dom[1].lat = value as u8;
                debug!("PI BSD DOM2 LAT: {}", self.bsd_dom[1].lat);
            }
            0x28 => {
                self.bsd_dom[1].pwd = value as u8;
                debug!("PI BSD DOM2 PWD: {}", self.bsd_dom[1].pwd);
            }
            0x2c => {
                self.bsd_dom[1].pgs = value as u8 & 15;
                debug!("PI BSD DOM2 PGS: {}", self.bsd_dom[1].pgs);
            }
            0x30 => {
                self.bsd_dom[1].rls = value as u8 & 3;
                debug!("PI BSD DOM2 RLS: {}", self.bsd_dom[1].rls);
            }
            _ => unimplemented!(
                "Peripheral Interface Write: {:08X} <= {:08X}",
                address,
                value
            ),
        }
    }
}

pub use pif::Pif;

use super::dma::DmaRequest;
use super::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::memory::{Masked, Reader, Writer};
use bitfield_struct::bitfield;

mod pif;

pub struct SerialInterface {
    dram_addr: u32,
    status: Status,
    rcp_int: RcpInterrupt,
    pif: Pif,
}

impl SerialInterface {
    pub fn new(rcp_int: RcpInterrupt) -> Self {
        Self {
            dram_addr: 0,
            status: Status::new(),
            rcp_int,
            pif: Pif::new(),
        }
    }

    pub fn pif(&self) -> &Pif {
        &self.pif
    }

    pub fn pif_mut(&mut self) -> &mut Pif {
        &mut self.pif
    }

    pub fn finish_dma(&mut self) {
        self.rcp_int.raise(RcpIntType::SI);
    }
}

impl Reader for SerialInterface {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        match address {
            0x00 => self.dram_addr,
            0x18 => {
                let mut status = self.status;
                status.set_interrupt(self.rcp_int.has(RcpIntType::SI));
                status.into()
            }
            _ => unimplemented!("Serial Interface Register Read: {:08X}", address),
        }
    }
}

impl Writer for SerialInterface {
    type SideEffect = Option<DmaRequest>;

    fn write_register(&mut self, address: u32, value: Masked<u32>) -> Option<DmaRequest> {
        match address {
            0x00 => value.write_reg_hex("SI_DRAM_ADDR", &mut self.dram_addr),
            0x04 => {
                self.pif.process();

                return Some(DmaRequest {
                    src: self.dram_addr & 0x00ff_ffff,
                    dst: value.get() & 0x07fc,
                    len: 64,
                    mode: true,
                });
            }
            0x10 => {
                return Some(DmaRequest {
                    src: self.dram_addr & 0x00ff_ffff,
                    dst: value.get() & 0x07fc,
                    len: 64,
                    mode: false,
                });
            }
            0x18 => self.rcp_int.clear(RcpIntType::SI),
            _ => unimplemented!("Serial Interface Register Write: {:08X}", address),
        }

        None
    }
}

#[bitfield(u32)]
struct Status {
    dma_busy: bool,
    io_busy: bool,
    read_pending: bool,
    dma_error: bool,
    #[bits(8)]
    __: u32,
    interrupt: bool,
    #[bits(19)]
    __: u32,
}

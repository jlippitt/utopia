use super::dma::DmaRequest;
use super::interrupt::{RcpIntType, RcpInterrupt};
use super::memory::{Masked, Reader, Writer};
use tracing::trace;

const PI_DRAM_ADDR: usize = 0;
const PI_CART_ADDR: usize = 1;
const PI_RD_LEN: usize = 2;
const PI_WR_LEN: usize = 3;
const PI_STATUS: usize = 4;
const PI_BSD_DOM1_LAT: usize = 5;
const PI_BSD_DOM1_PWD: usize = 6;
const PI_BSD_DOM1_PGS: usize = 7;
const PI_BSD_DOM1_RLS: usize = 8;

const REGS: [&str; 13] = [
    "PI_DRAM_ADDR",
    "PI_CART_ADDR",
    "PI_RD_LEN",
    "PI_WR_LEN",
    "PI_STATUS",
    "PI_BSD_DOM1_LAT",
    "PI_BSD_DOM1_PWD",
    "PI_BSD_DOM1_PGS",
    "PI_BSD_DOM1_RLS",
    "PI_BSD_DOM2_LAT",
    "PI_BSD_DOM2_PWD",
    "PI_BSD_DOM2_PGS",
    "PI_BSD_DOM2_RLS",
];

pub struct PeripheralInterface {
    regs: [u32; 13],
    rcp_int: RcpInterrupt,
}

impl PeripheralInterface {
    pub fn new(rcp_int: RcpInterrupt) -> Self {
        let mut regs = [0; 13];

        // These always read as 0x7f for some reason
        regs[PI_RD_LEN] = 0x7f;
        regs[PI_WR_LEN] = 0x7f;

        // TODO: Use values from cartridge header
        regs[PI_BSD_DOM1_LAT] = 64;
        regs[PI_BSD_DOM1_PWD] = 18;
        regs[PI_BSD_DOM1_PGS] = 7;
        regs[PI_BSD_DOM1_RLS] = 3;

        Self { regs, rcp_int }
    }

    pub fn finish_dma(&mut self) {
        self.rcp_int.raise(RcpIntType::PI);
    }
}

impl Reader for PeripheralInterface {
    fn read_u32(&self, address: u32) -> u32 {
        let index = (address as usize) >> 2;

        match index {
            PI_STATUS => {
                if self.rcp_int.has(RcpIntType::PI) {
                    0x08
                } else {
                    0
                }
            }
            _ => self.regs[(address as usize) >> 2],
        }
    }
}

impl Writer for PeripheralInterface {
    type SideEffect = Option<DmaRequest>;

    fn write_u32(&mut self, address: u32, value: Masked<u32>) -> Option<DmaRequest> {
        let index = (address as usize) >> 2;

        match index {
            PI_RD_LEN => {
                return Some(DmaRequest {
                    src: self.regs[PI_CART_ADDR] & 0xffff_fffe,
                    dst: self.regs[PI_DRAM_ADDR] & 0x00ff_fffe,
                    len: value.get() & 0x00ff_ffff,
                    mode: true,
                })
            }
            PI_WR_LEN => {
                return Some(DmaRequest {
                    src: self.regs[PI_CART_ADDR] & 0xffff_fffe,
                    dst: self.regs[PI_DRAM_ADDR] & 0x00ff_fffe,
                    len: value.get() & 0x00ff_ffff,
                    mode: false,
                })
            }
            PI_STATUS => {
                let input = value.get();

                if (input & 0x01) != 0 {
                    unimplemented!("PI DMA controller reset");
                }

                if (input & 0x02) != 0 {
                    self.rcp_int.clear(RcpIntType::PI);
                }
            }
            _ => {
                self.regs[index] = value.apply(self.regs[index]);
                trace!("{}: {:08X}", REGS[index], self.regs[index]);
            }
        }

        None
    }
}

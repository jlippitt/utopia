use super::super::dma::{Dma, DmaRequest};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::debug;

struct Inner {
    dma_requested: Dma,
    dma_spaddr: u32,
    dma_ramaddr: u32,
    single_step: bool,
}

#[derive(Clone)]
pub struct Registers {
    inner: Rc<RefCell<Inner>>,
}

impl Registers {
    pub const NAMES: [&str; 16] = [
        "DMA_CACHE",
        "DMA_DRAM",
        "DMA_READ_LENGTH",
        "DMA_WRITE_LENGTH",
        "SP_STATUS",
        "DMA_FULL",
        "DMA_BUSY",
        "SP_SEMAPHORE",
        "CMD_START",
        "CMD_END",
        "CMD_CURRENT",
        "CMD_STATUS",
        "CMD_CLOCK",
        "CMD_BUSY",
        "CMD_PIPE_BUSY",
        "CMD_TMEM_BUSY",
    ];

    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                dma_requested: Dma::None,
                dma_spaddr: 0,
                dma_ramaddr: 0,
                single_step: false,
            })),
        }
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
            4 => {
                // SP_STATUS
                // TODO
                0x01
            }
            5 => {
                // DMA_FULL
                // TODO
                0
            }
            6 => {
                // DMA_BUSY
                // TODO
                0
            }
            7 => {
                // SP_SEMAPHORE
                // TODO
                0
            }
            _ => unimplemented!("RSP CP0 Register Read: {}", Self::NAMES[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: u32) -> bool {
        let mut inner = self.inner.borrow_mut();

        match index {
            0 => {
                inner.dma_spaddr = value & 0x1ff8;
                debug!("SP_DMA_SPADDR: {:08X}", inner.dma_spaddr);
            }
            1 => {
                inner.dma_ramaddr = value & 0x00ff_fff8;
                debug!("SP_DMA_RAMADDR: {:08X}", inner.dma_ramaddr);
            }
            2 => {
                inner.dma_requested = Dma::Read(DmaRequest {
                    src_addr: inner.dma_spaddr,
                    dst_addr: inner.dma_ramaddr,
                    len: value & 0xff8f_fff8,
                })
            }
            4 => {
                // SP_STATUS
                // TODO
                if (value & 0x40) != 0 {
                    inner.single_step = true;
                    debug!("RSP Single Step: {}", inner.single_step);
                }

                if (value & 0x20) != 0 {
                    inner.single_step = false;
                    debug!("RSP Single Step: {}", inner.single_step);
                }

                if (value & 0x01) != 0 {
                    if inner.single_step {
                        todo!("Single step");
                    }

                    return true;
                }
            }
            7 => {
                // TODO: SP_SEMAPHORE
            }
            _ => unimplemented!(
                "RSP CP0 Register Write: {} <= {:08X}",
                Self::NAMES[index],
                value
            ),
        }

        false
    }

    pub fn dma_requested(&self) -> Dma {
        self.inner.borrow().dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.inner.borrow_mut().dma_requested = Dma::None;
    }
}

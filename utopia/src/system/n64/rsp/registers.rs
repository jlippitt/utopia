use super::super::dma::{Dma, DmaRequest};
use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u32)]
struct Status {
    halted: bool,
    broke: bool,
    dma_busy: bool,
    dma_full: bool,
    io_busy: bool,
    sstep: bool,
    intbreak: bool,
    sig: u8,
    #[bits(17)]
    __: u32,
}

pub struct Registers {
    dma_requested: Dma,
    dma_spaddr: u32,
    dma_ramaddr: u32,
    status: Status,
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
            dma_requested: Dma::None,
            dma_spaddr: 0,
            dma_ramaddr: 0,
            status: Status::new().with_halted(true),
        }
    }

    pub fn halted(&self) -> bool {
        self.status.halted()
    }

    pub fn single_step(&self) -> bool {
        self.status.sstep()
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
            4 => self.status.into(),
            5 => self.status.dma_full() as u32,
            6 => self.status.dma_busy() as u32,
            7 => {
                // SP_SEMAPHORE
                // TODO
                0
            }
            _ => unimplemented!("RSP CP0 Register Read: {}", Self::NAMES[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => {
                self.dma_spaddr = value & 0x1ff8;
                debug!("SP_DMA_SPADDR: {:08X}", self.dma_spaddr);
            }
            1 => {
                self.dma_ramaddr = value & 0x00ff_fff8;
                debug!("SP_DMA_RAMADDR: {:08X}", self.dma_ramaddr);
            }
            2 => {
                self.dma_requested = Dma::Read(DmaRequest {
                    src_addr: self.dma_spaddr,
                    dst_addr: self.dma_ramaddr,
                    len: value & 0xff8f_fff8,
                })
            }
            4 => {
                // SP_STATUS
                if (value & 0x01) != 0 {
                    self.status.set_halted(false)
                }

                if (value & 0x02) != 0 {
                    self.status.set_halted(true)
                }

                if (value & 0x04) != 0 {
                    self.status.set_broke(false)
                }

                if (value & 0x08) != 0 {
                    // TODO: Clear interrupt
                }

                if (value & 0x10) != 0 {
                    // TODO: Raise interrupt
                    todo!("RSP Interrupts");
                }

                if (value & 0x20) != 0 {
                    self.status.set_sstep(false)
                }

                if (value & 0x40) != 0 {
                    self.status.set_sstep(true)
                }

                if (value & 0x80) != 0 {
                    self.status.set_intbreak(false)
                }

                if (value & 0x100) != 0 {
                    self.status.set_intbreak(true)
                }

                let mut sig = self.status.sig();

                for bit in 0..=7 {
                    if (value & 0x200 << (bit << 1)) != 0 {
                        sig &= 1 << bit;
                    }

                    if (value & 0x400 << (bit << 1)) != 0 {
                        sig |= 1 << bit;
                    }
                }

                self.status.set_sig(sig);

                debug!("SP_STATUS: {:?}", self.status);
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
    }

    pub fn dma_requested(&self) -> Dma {
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = Dma::None;
    }
}

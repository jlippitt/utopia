use super::super::dma::Dma;
use super::super::interrupt::{RcpIntType, RcpInterrupt};
use super::super::rdp::RdpDma;
use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u32)]
struct RspStatus {
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

#[bitfield(u32)]
struct RdpStatus {
    xbus: bool,
    freeze: bool,
    flush: bool,
    start_gclk: bool,
    tmem_busy: bool,
    pipe_busy: bool,
    busy: bool,
    ready: bool,
    dma_busy: bool,
    end_pending: bool,
    start_pending: bool,
    #[bits(21)]
    __: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DmaType {
    None,
    Rsp(Dma),
    Rdp(RdpDma),
}

pub struct Registers {
    dma_requested: DmaType,
    sp_dma_spaddr: u32,
    sp_dma_ramaddr: u32,
    sp_status: RspStatus,
    dp_start: u32,
    dp_end: u32,
    dp_status: RdpStatus,
    interrupt: RcpInterrupt,
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

    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            dma_requested: DmaType::None,
            sp_dma_spaddr: 0,
            sp_dma_ramaddr: 0,
            sp_status: RspStatus::new().with_halted(true),
            dp_start: 0,
            dp_end: 0,
            dp_status: RdpStatus::new(),
            interrupt,
        }
    }

    pub fn halted(&self) -> bool {
        self.sp_status.halted()
    }

    pub fn single_step(&self) -> bool {
        self.sp_status.sstep()
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
            4 => self.sp_status.into(),
            5 => self.sp_status.dma_full() as u32,
            6 => self.sp_status.dma_busy() as u32,
            7 => {
                // SP_SEMAPHORE
                // TODO
                0
            }
            8 => self.dp_start,
            9 => self.dp_end,
            11 => self.dp_status.into(),
            _ => unimplemented!("RSP CP0 Register Read: {}", Self::NAMES[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => {
                self.sp_dma_spaddr = value & 0x1ff8;
                debug!("SP_DMA_SPADDR: {:08X}", self.sp_dma_spaddr);
            }
            1 => {
                self.sp_dma_ramaddr = value & 0x00ff_fff8;
                debug!("SP_DMA_RAMADDR: {:08X}", self.sp_dma_ramaddr);
            }
            2 => {
                self.dma_requested = DmaType::Rsp(Dma {
                    src_addr: self.sp_dma_spaddr,
                    dst_addr: self.sp_dma_ramaddr,
                    len: value & 0xff8f_fff8,
                    reverse: true,
                });
            }
            3 => {
                self.dma_requested = DmaType::Rsp(Dma {
                    src_addr: self.sp_dma_spaddr,
                    dst_addr: self.sp_dma_ramaddr,
                    len: value & 0xff8f_fff8,
                    reverse: false,
                });
            }
            4 => {
                // SP_STATUS
                if (value & 0x01) != 0 {
                    self.sp_status.set_halted(false)
                }

                if (value & 0x02) != 0 {
                    self.sp_status.set_halted(true)
                }

                if (value & 0x04) != 0 {
                    self.sp_status.set_broke(false)
                }

                if (value & 0x08) != 0 {
                    self.interrupt.clear(RcpIntType::SP);
                }

                if (value & 0x10) != 0 {
                    self.interrupt.raise(RcpIntType::SP);
                }

                if (value & 0x20) != 0 {
                    self.sp_status.set_sstep(false)
                }

                if (value & 0x40) != 0 {
                    self.sp_status.set_sstep(true)
                }

                if (value & 0x80) != 0 {
                    self.sp_status.set_intbreak(false)
                }

                if (value & 0x100) != 0 {
                    self.sp_status.set_intbreak(true)
                }

                let mut sig = self.sp_status.sig();

                for bit in 0..=7 {
                    if (value & 0x200 << (bit << 1)) != 0 {
                        sig &= 1 << bit;
                    }

                    if (value & 0x400 << (bit << 1)) != 0 {
                        sig |= 1 << bit;
                    }
                }

                self.sp_status.set_sig(sig);

                debug!("SP_STATUS: {:?}", self.sp_status);
            }
            7 => {
                // TODO: SP_SEMAPHORE
            }
            8 => {
                self.dp_start = value & 0x00ff_ffff;
                debug!("DP_START: {:08X}", self.dp_start);
                self.dp_status.set_start_pending(true);
            }
            9 => {
                self.dp_end = value & 0x00ff_ffff;
                debug!("DP_END: {:08X}", self.dp_end);
                self.dp_status.set_start_pending(false);

                self.dma_requested = DmaType::Rdp(RdpDma {
                    start: self.dp_start,
                    end: self.dp_end,
                })
            }
            _ => unimplemented!(
                "RSP CP0 Register Write: {} <= {:08X}",
                Self::NAMES[index],
                value
            ),
        }
    }

    pub fn is_done(&self) -> bool {
        self.dma_requested != DmaType::None || self.halted()
    }

    pub fn break_(&mut self) {
        self.sp_status.set_halted(true);
        self.sp_status.set_broke(true);

        if self.sp_status.intbreak() {
            self.interrupt.raise(RcpIntType::SP);
        }
    }

    pub fn dma_requested(&self) -> DmaType {
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = DmaType::None;
    }
}

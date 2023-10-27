use crate::system::n64::dma::DmaRequest;
use crate::system::n64::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::memory::Masked;
use bitfield_struct::bitfield;
use std::cell::Cell;
use tracing::trace;

#[derive(Debug, Default)]
pub enum DmaType {
    #[default]
    None,
    Rsp(DmaRequest),
    Rdp(DmaRequest),
}

pub struct Registers {
    running: bool,
    rcp_int: RcpInterrupt,
    dma_type: DmaType,
    sp_dma_spaddr: u32,
    sp_dma_ramaddr: u32,
    sp_status: RspStatus,
    sp_semaphore: Cell<bool>,
    dp_start: u32,
    dp_end: u32,
    dp_current: u32,
    dp_status: RdpStatus,
}

impl Registers {
    pub const NAMES: [&str; 16] = [
        "SP_DMA_SPADDR",
        "SP_DMA_RAMADDR",
        "SP_DMA_RDLEN",
        "SP_DMA_WRLEN",
        "SP_STATUS",
        "SP_DMA_FULL",
        "SP_DMA_BUSY",
        "SP_SEMAPHORE",
        "DP_START",
        "DP_END",
        "DP_CURRENT",
        "DP_STATUS",
        "DP_CLOCK",
        "DPC_BUSY",
        "DPC_PIPE_BUSY",
        "DPC_TMEM_BUSY",
    ];

    pub fn new(rcp_int: RcpInterrupt) -> Self {
        Self {
            running: false,
            rcp_int,
            dma_type: DmaType::None,
            sp_dma_spaddr: 0,
            sp_dma_ramaddr: 0,
            sp_status: RspStatus::new().with_halted(true),
            sp_semaphore: Cell::new(false),
            dp_start: 0,
            dp_end: 0,
            dp_current: 0,
            dp_status: RdpStatus::new(),
        }
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
            0 => self.sp_dma_spaddr,
            1 => self.sp_dma_ramaddr,
            2 | 3 => 0xff8,
            4 => self.sp_status.into(),
            5 => self.sp_status.dma_full() as u32,
            6 => self.sp_status.dma_busy() as u32,
            7 => {
                let value = self.sp_semaphore.get();
                self.sp_semaphore.set(true);
                trace!("SP_SEMAPHORE: {}", self.sp_semaphore.get());
                value as u32
            }
            8 => self.dp_start,
            9 => self.dp_end,
            10 => self.dp_current,
            11 => self.dp_status.into(),
            _ => panic!("Unmapped RSP CP0 Register Read: {}", Self::NAMES[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: Masked<u32>) {
        match index {
            0 => value.write_reg_hex("SP_DMA_SPADDR", &mut self.sp_dma_spaddr),
            1 => value.write_reg_hex("SP_DMA_RAMADDR", &mut self.sp_dma_ramaddr),
            2 => {
                self.dma_type = DmaType::Rsp(DmaRequest {
                    src: self.sp_dma_spaddr,
                    dst: self.sp_dma_ramaddr,
                    len: value.get(),
                    mode: true,
                });

                self.running = false;
            }
            3 => {
                self.dma_type = DmaType::Rsp(DmaRequest {
                    src: self.sp_dma_spaddr,
                    dst: self.sp_dma_ramaddr,
                    len: value.get(),
                    mode: false,
                });

                self.running = false;
            }
            4 => {
                let input = value.get();

                if (input & 0x01) != 0 {
                    self.sp_status.set_halted(false);
                }

                if (input & 0x02) != 0 {
                    self.sp_status.set_halted(true);
                }

                if (input & 0x04) != 0 {
                    self.sp_status.set_broke(false);
                }

                if (input & 0x08) != 0 {
                    self.rcp_int.clear(RcpIntType::SP);
                }

                if (input & 0x10) != 0 {
                    self.rcp_int.raise(RcpIntType::SP);
                }

                if (input & 0x20) != 0 {
                    self.sp_status.set_sstep(false);
                }

                if (input & 0x40) != 0 {
                    self.sp_status.set_sstep(true);
                }

                if (input & 0x80) != 0 {
                    self.sp_status.set_intbreak(false);
                }

                if (input & 0x100) != 0 {
                    self.sp_status.set_intbreak(true);
                }

                {
                    let mut signal = self.sp_status.signal();

                    for index in 0..8 {
                        let shift = 9 + (index << 1);

                        if (input & (1 << shift)) != 0 {
                            signal &= !(1 << index);
                        }

                        if (input & (1 << (shift + 1))) != 0 {
                            signal |= 1 << index;
                        }
                    }

                    self.sp_status.set_signal(signal);
                }

                trace!("SP_STATUS: {:?}", self.sp_status);

                self.try_restart();
            }
            7 => {
                self.sp_semaphore.set((value.get() & 1) != 0);
                trace!("SP_SEMAPHORE: {}", self.sp_semaphore.get());
            }
            8 => {
                if !self.dp_status.start_pending() {
                    value.write_reg_hex("DP_START", &mut self.dp_start);
                    self.dp_start &= 0x00ff_fff8;
                    self.dp_status.set_start_pending(true);
                }
            }
            9 => {
                value.write_reg_hex("DP_END", &mut self.dp_end);
                self.dp_end &= 0x00ff_fff8;
                self.dp_status.set_buffer_busy(true);

                debug_assert!(matches!(self.dma_type, DmaType::None));

                let start = if self.dp_status.start_pending() {
                    self.dp_status.set_start_pending(false);
                    self.dp_start
                } else {
                    self.dp_current
                };

                if self.dp_end > start {
                    self.dma_type = DmaType::Rdp(DmaRequest {
                        src: start,
                        dst: 0,
                        len: self.dp_end - start,
                        mode: self.dp_status.xbus(),
                    });

                    self.set_dp_ready(false);
                    self.running = false;
                }

                self.dp_current = self.dp_end;
            }
            11 => {
                let input = value.get();

                if (input & 0x0001) != 0 {
                    self.dp_status.set_xbus(false);
                }

                if (input & 0x0002) != 0 {
                    self.dp_status.set_xbus(true);
                }

                if (input & 0x0004) != 0 {
                    self.dp_status.set_freeze(false);
                }

                if (input & 0x0008) != 0 {
                    self.dp_status.set_freeze(true);
                }

                if (input & 0x0010) != 0 {
                    self.dp_status.set_flush(false);
                }

                if (input & 0x0020) != 0 {
                    self.dp_status.set_flush(true);
                }

                if (input & 0x0040) != 0 {
                    self.dp_status.set_tmem_busy(false);
                }

                if (input & 0x0080) != 0 {
                    self.dp_status.set_pipe_busy(false);
                }

                if (input & 0x0100) != 0 {
                    self.dp_status.set_buffer_busy(false);
                }

                if (input & 0x0200) != 0 {
                    // TODO: DP_CLOCK
                }
            }
            _ => panic!("Unmapped RSP CP0 Register Write: {}", Self::NAMES[index]),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_single_step(&self) -> bool {
        self.sp_status.sstep()
    }

    pub fn take_dma_type(&mut self) -> DmaType {
        self.running = !self.sp_status.halted() && !self.rcp_int.has(RcpIntType::SP);
        std::mem::take(&mut self.dma_type)
    }

    pub fn finish_rsp_dma(&mut self, sp_dma_spaddr: u32, sp_dma_ramaddr: u32) {
        self.sp_dma_spaddr = sp_dma_spaddr;
        self.sp_dma_ramaddr = sp_dma_ramaddr;
    }

    pub fn try_restart(&mut self) -> bool {
        self.running = !self.sp_status.halted()
            && !self.rcp_int.has(RcpIntType::SP)
            && matches!(self.dma_type, DmaType::None);

        self.running
    }

    pub fn halt(&mut self) {
        self.sp_status.set_halted(true);
        self.running = false;
    }

    pub fn break_(&mut self) {
        self.halt();
        self.sp_status.set_broke(true);

        if self.sp_status.intbreak() {
            self.rcp_int.raise(RcpIntType::SP);
        }
    }

    pub fn set_dp_ready(&mut self, ready: bool) {
        self.dp_status.set_ready(ready);
    }

    pub fn clear_buffer_busy(&mut self) {
        self.dp_status.set_buffer_busy(false);
    }
}

#[bitfield(u32)]
pub struct RspStatus {
    pub halted: bool,
    pub broke: bool,
    pub dma_busy: bool,
    pub dma_full: bool,
    pub io_busy: bool,
    pub sstep: bool,
    pub intbreak: bool,
    pub signal: u8,
    #[bits(17)]
    __: u32,
}

#[bitfield(u32)]
pub struct RdpStatus {
    pub xbus: bool,
    pub freeze: bool,
    pub flush: bool,
    pub start_gclk: bool,
    pub tmem_busy: bool,
    pub pipe_busy: bool,
    pub buffer_busy: bool,
    pub ready: bool,
    pub dma_busy: bool,
    pub end_pending: bool,
    pub start_pending: bool,
    #[bits(21)]
    __: u32,
}

use super::super::dma::Dma;
use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u32)]
struct Status {
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

pub struct Registers {
    dma_requested: Option<Dma>,
    start: u32,
    end: u32,
    status: Status,
}

impl Registers {
    pub const NAMES: [&str; 8] = [
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
            dma_requested: None,
            start: 0,
            end: 0,
            status: Status::new(),
        }
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
            0 => self.start,
            1 => self.end,
            3 => self.status.into(),
            _ => unimplemented!("RDP Command Register Read: {}", Self::NAMES[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            0 => {
                self.start = value & 0x00ff_ffff;
                debug!("DP_START: {:08X}", self.start);
                self.status.set_start_pending(true);
            }
            1 => {
                self.end = value & 0x00ff_ffff;
                debug!("DP_END: {:08X}", self.end);
                self.status.set_start_pending(false);

                self.dma_requested = Some(Dma {
                    src_addr: self.start,
                    dst_addr: 0, // Not used
                    len: self.end - self.start,
                    reverse: false, // Not used
                })
            }
            _ => unimplemented!(
                "RDP Command Register Write: {} <= {:08X}",
                Self::NAMES[index],
                value
            ),
        }
    }
}

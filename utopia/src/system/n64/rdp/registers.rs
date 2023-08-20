use bitfield_struct::bitfield;

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
            status: Status::new(),
        }
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
            3 => self.status.into(),
            _ => unimplemented!("RDP Command Register Read: {}", Self::NAMES[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: u32) {
        match index {
            _ => unimplemented!(
                "RDP Command Register Write: {} <= {:08X}",
                Self::NAMES[index],
                value
            ),
        }
    }
}

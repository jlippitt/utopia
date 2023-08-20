pub struct Registers {
    //
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
        Self {}
    }

    pub fn get(&self, index: usize) -> u32 {
        match index {
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

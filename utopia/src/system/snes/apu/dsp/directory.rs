use tracing::debug;

pub struct Directory {
    base_address: u16,
}

impl Directory {
    pub fn new() -> Self {
        Self { base_address: 0 }
    }

    pub fn set_base_address(&mut self, value: u8) {
        self.base_address = (value as u16) << 8;
        debug!("DIR Base Address: {:04X}", self.base_address);
    }
}

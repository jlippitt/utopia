use tracing::{debug, warn};

pub struct Oam {
    address: u8,
}

impl Oam {
    pub fn new() -> Self {
        Self { address: 0 }
    }

    pub fn set_address(&mut self, value: u8) {
        self.address = value;
        debug!("OAM Address: {:02X}", self.address);
    }

    pub fn read(&mut self) -> u8 {
        panic!("OAM reads not yet implemented");
    }

    pub fn write(&mut self, _value: u8) {
        warn!("OAM writes not yet implemented");
    }
}

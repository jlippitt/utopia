use tracing::debug;

pub struct Oam {
    address: u8,
    data: [u8; 256],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            address: 0,
            data: [0; 256],
        }
    }

    pub fn set_address(&mut self, value: u8) {
        self.address = value;
        debug!("OAM Address: {:02X}", self.address);
    }

    pub fn read(&self) -> u8 {
        let value = self.data[self.address as usize];
        debug!("OAM Read: {:02X} => {:02X}", self.address, value);
        value
    }

    pub fn write(&mut self, value: u8) {
        debug!("OAM Write: {:02X} <= {:02X}", self.address, value);
        self.data[self.address as usize] = value;
        self.address = self.address.wrapping_add(1);
    }
}

use tracing::warn;

pub struct Vdc {
    //
}

impl Vdc {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, address: u16, _prev_value: u8) -> u8 {
        unimplemented!("VDC Read: {:04X}", address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        warn!("Unimplemented VDC Write: {:04X} <= {:02X}", address, value);
    }
}

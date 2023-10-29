use tracing::warn;

pub struct Vdp {
    //
}

impl Vdp {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write_command(&mut self, value: u8) {
        warn!("VDP Command: {:02X}", value);
    }

    pub fn write_data(&mut self, value: u8) {
        warn!("VDP Data: {:02X}", value);
    }
}

use tracing::debug;

pub struct Palette {
    data: [u8; 32],
}

impl Palette {
    pub fn new() -> Self {
        Self {
            data: [0; 32],
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let mask = if (address & 0x03) == 0 { 0x0f } else { 0x1f };
        let index = address as usize & mask;
        self.data[index] = value & 0x3f;
        debug!("Palette Write: {:02X} <= {:02X}", index, self.data[index]);
    }
}
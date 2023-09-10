use tracing::debug;

pub struct SpriteLayer {
    enabled: bool,
    table_address: u16,
}

impl SpriteLayer {
    pub fn new() -> Self {
        Self {
            enabled: false,
            table_address: 0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("Sprite Layer Enabled: {}", self.enabled);
    }

    pub fn set_table_address(&mut self, msb: bool, value: u8) {
        self.table_address = if msb {
            (self.table_address & 0xff) | ((value as u16) << 8)
        } else {
            (self.table_address & 0xff00) | value as u16
        };

        debug!("Sprite Table Address: {:04X}", self.table_address);
    }
}

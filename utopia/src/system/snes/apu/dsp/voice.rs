use tracing::debug;

pub struct Voice {
    pitch: i32,
    source: u8,
    id: u32,
}

impl Voice {
    pub fn new(id: u32) -> Self {
        Self {
            pitch: 0,
            source: 0,
            id: id,
        }
    }

    pub fn envelope(&self) -> u8 {
        // TODO
        0
    }

    pub fn output(&self) -> u8 {
        // TODO
        0
    }

    pub fn set_volume_left(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_volume_right(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_pitch_low(&mut self, value: u8) {
        self.pitch = (self.pitch & 0x3f00) | (value as i32);
        debug!("V{} Pitch: {:04X}", self.id, self.pitch);
    }

    pub fn set_pitch_high(&mut self, value: u8) {
        self.pitch = (self.pitch & 0xff) | ((value as i32 & 0x3f) << 8);
        debug!("V{} Pitch: {:04X}", self.id, self.pitch);
    }

    pub fn set_source(&mut self, value: u8) {
        self.source = value;
        debug!("V{} Source: {:02X}", self.id, self.source);
    }

    pub fn set_adsr_low(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_adsr_high(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_gain(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_key_on(&mut self, _key_on: bool) {
        // TODO
    }

    pub fn set_key_off(&mut self, _key_off: bool) {
        // TODO
    }
}

pub struct Voice {
    _id: u32,
}

impl Voice {
    pub fn new(id: u32) -> Self {
        Self { _id: id }
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

    pub fn set_pitch_low(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_pitch_high(&mut self, _value: u8) {
        // TODO
    }

    pub fn set_source(&mut self, _value: u8) {
        // TODO
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

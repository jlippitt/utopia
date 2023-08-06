use super::directory::Directory;
use crate::util::MirrorVec;
use decoder::BrrDecoder;
use tracing::debug;

mod decoder;

pub struct Voice {
    pitch: i32,
    source: u8,
    key_on: bool,
    decoder: BrrDecoder,
    id: u32,
}

impl Voice {
    pub fn new(id: u32) -> Self {
        Self {
            pitch: 0,
            source: 0,
            key_on: false,
            decoder: BrrDecoder::new(),
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
        debug!("Voice {} Pitch: {:04X}", self.id, self.pitch);
    }

    pub fn set_pitch_high(&mut self, value: u8) {
        self.pitch = (self.pitch & 0xff) | ((value as i32 & 0x3f) << 8);
        debug!("Voice {} Pitch: {:04X}", self.id, self.pitch);
    }

    pub fn set_source(&mut self, value: u8) {
        self.source = value;
        debug!("Voice {} Source: {:02X}", self.id, self.source);
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

    pub fn set_key_on(&mut self, key_on: bool) {
        self.key_on = key_on;
        debug!("Voice {} Key On: {}", self.id, self.key_on);
    }

    pub fn set_key_off(&mut self, _key_off: bool) {
        // TODO
    }

    pub fn step(
        &mut self,
        dir: &Directory,
        ram: &MirrorVec<u8>,
        poll_key_state: bool,
    ) -> (i32, i32) {
        debug!("Voice {} Begin", self.id);

        if poll_key_state && self.key_on {
            debug!("Voice {} Restarting", self.id);
            self.key_on = false;
            let start_address = dir.start_address(ram, self.source);
            self.decoder.restart(ram, start_address);
            // TODO: Should be a 5 sample delay?
        }

        // TODO: Key off

        debug!("Voice {} End", self.id);

        // TODO
        (0, 0)
    }
}

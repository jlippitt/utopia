use super::directory::Directory;
use crate::util::MirrorVec;
use decoder::{BlockMode, BrrDecoder};
use tracing::debug;

mod decoder;

pub struct Voice {
    pitch: usize,
    source: u8,
    key_on: bool,
    counter: usize,
    decoder: BrrDecoder,
    id: u32,
}

impl Voice {
    pub fn new(id: u32) -> Self {
        Self {
            pitch: 0,
            source: 0,
            key_on: false,
            counter: 0,
            decoder: BrrDecoder::new(id),
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
        self.pitch = (self.pitch & 0x3f00) | (value as usize);
        debug!("Voice {} Pitch: {:04X}", self.id, self.pitch);
    }

    pub fn set_pitch_high(&mut self, value: u8) {
        self.pitch = (self.pitch & 0xff) | ((value as usize & 0x3f) << 8);
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
        if poll_key_state && self.key_on {
            self.key_on = false;
            self.counter = 0;

            let start_address = dir.start_address(ram, self.source);
            debug!("Voice {} Start Address: {:04X}", self.id, start_address);
            self.decoder.restart(ram, start_address);

            // TODO: Should be a 5 sample delay?
        } else {
            // TODO: Pitch modulation
            self.counter += self.pitch;

            if self.counter > 0xffff {
                self.counter &= 0xffff;
                self.next_block(dir, ram);
            }
        }

        // TODO: Key off

        let sample = self.decoder.sample(self.counter);

        // TODO: Volume & Envelope

        debug!("Voice {} Output: {}", self.id, sample);

        (sample, sample)
    }

    fn next_block(&mut self, dir: &Directory, ram: &MirrorVec<u8>) {
        if self.decoder.block_mode() != BlockMode::Normal {
            // TODO: Set END flag
            debug!("Voice {} End", self.id);

            if self.decoder.block_mode() == BlockMode::EndMute {
                // TODO: Mute
                debug!("Voice {} Muted", self.id);
            }

            let loop_address = dir.loop_address(ram, self.source);
            debug!("Voice {} Loop Address: {:04X}", self.id, loop_address);
            self.decoder.restart(ram, loop_address);
        } else {
            self.decoder.decode_next(ram);
        }
    }
}

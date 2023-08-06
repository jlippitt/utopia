use super::directory::Directory;
use crate::util::MirrorVec;
use decoder::{BrrDecoder, LoopMode};
use envelope::Envelope;
use tracing::{debug, warn};

mod decoder;
mod envelope;

pub struct Voice {
    volume_left: i32,
    volume_right: i32,
    pitch: usize,
    source: u8,
    envelope: Envelope,
    key_on: bool,
    key_off: bool,
    noise_enabled: bool,
    echo_enabled: bool,
    counter: usize,
    decoder: BrrDecoder,
    id: u32,
}

impl Voice {
    pub fn new(id: u32) -> Self {
        Self {
            volume_left: 0,
            volume_right: 0,
            pitch: 0,
            source: 0,
            envelope: Envelope::new(id),
            key_on: false,
            key_off: false,
            noise_enabled: false,
            echo_enabled: false,
            counter: 0,
            decoder: BrrDecoder::new(id),
            id: id,
        }
    }

    pub fn envelope(&self) -> u8 {
        (self.envelope.level() >> 4) as u8
    }

    pub fn output(&self) -> u8 {
        warn!("Output read not yet implemented");
        0
    }

    pub fn echo_enabled(&self) -> bool {
        self.echo_enabled
    }

    pub fn set_volume_left(&mut self, value: u8) {
        self.volume_left = value as i8 as i32;
        debug!("Voice {} Volume Left: {}", self.id, self.volume_left);
    }

    pub fn set_volume_right(&mut self, value: u8) {
        self.volume_right = value as i8 as i32;
        debug!("Voice {} Volume Right: {}", self.id, self.volume_right);
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

    pub fn set_adsr_low(&mut self, value: u8) {
        self.envelope.set_adsr_low(value)
    }

    pub fn set_adsr_high(&mut self, value: u8) {
        self.envelope.set_adsr_high(value)
    }

    pub fn set_gain(&mut self, value: u8) {
        self.envelope.set_gain(value)
    }

    pub fn set_key_on(&mut self, key_on: bool) {
        self.key_on = key_on;
        debug!("Voice {} Key On: {}", self.id, self.key_on);
    }

    pub fn set_key_off(&mut self, key_off: bool) {
        self.key_off = key_off;
        debug!("Voice {} Key Off: {}", self.id, self.key_off);
    }

    pub fn set_noise_enabled(&mut self, noise_enabled: bool) {
        self.noise_enabled = noise_enabled;
        debug!("Voice {} Noise Enabled: {}", self.id, self.noise_enabled);
    }

    pub fn set_echo_enabled(&mut self, echo_enabled: bool) {
        self.echo_enabled = echo_enabled;
        debug!("Voice {} Echo Enabled: {}", self.id, self.echo_enabled);
    }

    pub fn step(
        &mut self,
        dir: &Directory,
        ram: &MirrorVec<u8>,
        noise_level: i32,
        poll_key_state: bool,
    ) -> (i32, i32) {
        if poll_key_state && self.key_on {
            self.key_on = false;
            self.counter = 0;

            let start_address = dir.start_address(ram, self.source);
            debug!("Voice {} Start Address: {:04X}", self.id, start_address);
            self.decoder.restart(ram, start_address);

            self.envelope.restart();

            // TODO: Should be a 5 sample delay?
        } else {
            // TODO: Pitch modulation
            self.counter += self.pitch;

            if self.counter > 0xffff {
                self.counter &= 0xffff;
                self.next_block(dir, ram);
            }
        }

        if poll_key_state && self.key_off {
            self.key_off = false;
            self.envelope.release();
        }

        self.envelope.step();

        let sample = if self.noise_enabled {
            noise_level
        } else {
            self.decoder.sample(self.counter)
        };

        let output = (sample * self.envelope.level()) >> 11;

        let left = (output * self.volume_left) >> 6;
        let right = (output * self.volume_right) >> 6;

        debug!("Voice {} Output: ({}, {})", self.id, left, right);

        (left, right)
    }

    fn next_block(&mut self, dir: &Directory, ram: &MirrorVec<u8>) {
        if self.decoder.loop_mode() != LoopMode::Normal {
            // TODO: Set END flag
            debug!("Voice {} End", self.id);

            if self.decoder.loop_mode() == LoopMode::EndMute {
                self.envelope.mute();
            }

            let loop_address = dir.loop_address(ram, self.source);
            debug!("Voice {} Loop Address: {:04X}", self.id, loop_address);
            self.decoder.restart(ram, loop_address);
        } else {
            self.decoder.decode_next(ram);
        }
    }
}

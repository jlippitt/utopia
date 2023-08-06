use crate::util::MirrorVec;
use crate::AudioQueue;
use directory::Directory;
use tracing::debug;
use voice::Voice;

mod directory;
mod voice;

const TOTAL_REGISTERS: usize = 128;

pub struct Dsp {
    address: u8,
    poll_key_state: bool,
    volume_left: i32,
    volume_right: i32,
    dir: Directory,
    voices: [Voice; 8],
    audio_queue: AudioQueue,
    data: [u8; TOTAL_REGISTERS],
}

impl Dsp {
    pub fn new() -> Self {
        Self {
            address: 0,
            poll_key_state: false,
            volume_left: 0,
            volume_right: 0,
            dir: Directory::new(),
            voices: [
                Voice::new(0),
                Voice::new(1),
                Voice::new(2),
                Voice::new(3),
                Voice::new(4),
                Voice::new(5),
                Voice::new(6),
                Voice::new(7),
            ],
            audio_queue: AudioQueue::new(),
            data: [0; TOTAL_REGISTERS],
        }
    }

    pub fn address(&self) -> u8 {
        self.address
    }

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        &mut self.audio_queue
    }

    pub fn set_address(&mut self, value: u8) {
        self.address = value;
        debug!("DSP Address: {:02X}", self.address);
    }

    pub fn read(&self) -> u8 {
        let address = self.address & 0x7f;

        let value = match self.address & 0x0f {
            0x08 => self.voice(address).envelope(),
            0x09 => self.voice(address).output(),
            _ => {
                if address == 0x7c {
                    // TODO: ENDX
                    0
                } else {
                    self.data[address as usize]
                }
            }
        };

        debug!("DSP Read: {:02X} >= {:02X}", address, value);

        value
    }

    pub fn write(&mut self, value: u8) {
        if self.address > 0x7f {
            return;
        }

        self.data[self.address as usize] = value;
        debug!("DSP Write: {:02X} <= {:02X}", self.address, value);

        match self.address & 0x0f {
            0x00 => self.voice_mut(self.address).set_volume_left(value),
            0x01 => self.voice_mut(self.address).set_volume_right(value),
            0x02 => self.voice_mut(self.address).set_pitch_low(value),
            0x03 => self.voice_mut(self.address).set_pitch_high(value),
            0x04 => self.voice_mut(self.address).set_source(value),
            0x05 => self.voice_mut(self.address).set_adsr_low(value),
            0x06 => self.voice_mut(self.address).set_adsr_high(value),
            0x07 => self.voice_mut(self.address).set_gain(value),
            0x08 => (), // TODO: ENVX (read-only?)
            0x09 => (), // TODO: OUTX (read-only?)
            0x0c => {
                match self.address {
                    0x0c => {
                        self.volume_left = value as i8 as i32;
                        debug!("DSP Volume Left: {}", self.volume_left);
                    }
                    0x1c => {
                        self.volume_right = value as i8 as i32;
                        debug!("DSP Volume Right: {}", self.volume_right);
                    }
                    0x2c => (), // TODO: Echo volume (left)
                    0x3c => (), // TODO: Echo volume (right)
                    0x4c => self.write_all(value, |voice, bit| voice.set_key_on(bit)),
                    0x5c => self.write_all(value, |voice, bit| voice.set_key_off(bit)),
                    0x6c => (), // TODO: Flags
                    0x7c => (), // TODO: ENDX (read-only?)
                    _ => unreachable!(),
                }
            }
            0x0d => {
                match self.address {
                    0x0d => (), // TODO: Echo feedback
                    0x1d => (), // TODO: Not used
                    0x2d => (), // TODO: Pitch modulation
                    0x3d => (), // TODO: Noise enable
                    0x4d => (), // TOOD: Echo enable
                    0x5d => self.dir.set_base_address(value),
                    0x6d => (), // TODO: Echo start address
                    0x7d => (), // TODO: Echo delay
                    _ => unreachable!(),
                }
            }
            0x0f => {
                // TODO: FIR coefficients
            }
            _ => (), // TODO: Not used
        }
    }

    pub fn step(&mut self, ram: &MirrorVec<u8>) {
        debug!("DSP Step Begin");

        let (mut left, mut right) = (0, 0);

        for voice in &mut self.voices {
            let sample = voice.step(&self.dir, ram, self.poll_key_state);
            left = clamp16(left + sample.0);
            right = clamp16(right + sample.1);
        }

        left = clamp16((left * self.volume_left) >> 7);
        right = clamp16((right * self.volume_right) >> 7);

        // TODO: Echo
        // TODO: Mute

        self.audio_queue
            .push_back((!left as f32 / 32768.0, !right as f32 / 32768.0));

        self.poll_key_state = !self.poll_key_state;

        debug!("DSP Step End");
    }

    fn voice(&self, address: u8) -> &Voice {
        &self.voices[(address >> 4) as usize]
    }

    fn voice_mut(&mut self, address: u8) -> &mut Voice {
        &mut self.voices[(address >> 4) as usize]
    }

    fn write_all(&mut self, value: u8, callback: impl Fn(&mut Voice, bool)) {
        for (index, voice) in self.voices.iter_mut().enumerate() {
            let bit = ((value >> index) & 1) != 0;
            callback(voice, bit);
        }
    }
}

fn clamp16(value: i32) -> i32 {
    value.clamp(u16::MIN as i32, u16::MAX as i32)
}

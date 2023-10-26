use crate::util::MirrorVec;
use crate::AudioQueue;
use directory::Directory;
use echo::Echo;
use noise::NoiseGenerator;
use tracing::{trace, warn};
use voice::Voice;

mod constants;
mod directory;
mod echo;
mod noise;
mod voice;

const TOTAL_REGISTERS: usize = 128;

pub struct Dsp {
    address: u8,
    poll_key_state: bool,
    volume_left: i32,
    volume_right: i32,
    echo: Echo,
    noise: NoiseGenerator,
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
            noise: NoiseGenerator::new(),
            echo: Echo::new(),
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
        trace!("DSP Address: {:02X}", self.address);
    }

    pub fn read(&self) -> u8 {
        let address = self.address & 0x7f;

        let value = match self.address & 0x0f {
            0x08 => self.voice(address).envelope(),
            0x09 => self.voice(address).output(),
            _ => {
                if address == 0x7c {
                    warn!("ENDX read not yet implemented");
                    0
                } else {
                    self.data[address as usize]
                }
            }
        };

        trace!("DSP Read: {:02X} >= {:02X}", address, value);

        value
    }

    pub fn write(&mut self, value: u8) {
        if self.address > 0x7f {
            return;
        }

        self.data[self.address as usize] = value;
        trace!("DSP Write: {:02X} <= {:02X}", self.address, value);

        match self.address & 0x0f {
            0x00 => self.voice_mut(self.address).set_volume_left(value),
            0x01 => self.voice_mut(self.address).set_volume_right(value),
            0x02 => self.voice_mut(self.address).set_pitch_low(value),
            0x03 => self.voice_mut(self.address).set_pitch_high(value),
            0x04 => self.voice_mut(self.address).set_source(value),
            0x05 => self.voice_mut(self.address).set_adsr_low(value),
            0x06 => self.voice_mut(self.address).set_adsr_high(value),
            0x07 => self.voice_mut(self.address).set_gain(value),
            0x08 => warn!("ENVX Write: {:02X} <= {:02X}", self.address, value),
            0x09 => warn!("OUTX Write: {:02X} <= {:02X}", self.address, value),
            0x0c => match self.address {
                0x0c => {
                    self.volume_left = value as i8 as i32;
                    trace!("DSP Volume Left: {}", self.volume_left);
                }
                0x1c => {
                    self.volume_right = value as i8 as i32;
                    trace!("DSP Volume Right: {}", self.volume_right);
                }
                0x2c => self.echo.set_volume_left(value),
                0x3c => self.echo.set_volume_right(value),
                0x4c => self.write_all(value, |voice, bit| voice.set_key_on(bit)),
                0x5c => self.write_all(value, |voice, bit| voice.set_key_off(bit)),
                0x6c => {
                    self.noise.set_rate(value & 0x1f);
                    self.echo.set_write_enabled((value & 0x20) == 0);

                    if (value & 0xc0) != 0 {
                        warn!("Flag write not yet implemented: {:02X}", value);
                    }
                }
                0x7c => warn!("ENDX Write: {:02X}", value),
                _ => unreachable!(),
            },
            0x0d => {
                match self.address {
                    0x0d => self.echo.set_feedback_volume(value),
                    0x1d => (), // TODO: Not used
                    0x2d => {
                        if value != 0 {
                            warn!("Pitch modulation not yet implemented");
                        }
                    }
                    0x3d => self.write_all(value, |voice, bit| voice.set_noise_enabled(bit)),
                    0x4d => self.write_all(value, |voice, bit| voice.set_echo_enabled(bit)),
                    0x5d => self.dir.set_base_address(value),
                    0x6d => self.echo.set_base_address(value),
                    0x7d => self.echo.set_buffer_size(value),
                    _ => unreachable!(),
                }
            }
            0x0f => self.echo.set_fir_value((self.address >> 4) as usize, value),
            _ => (), // TODO: Not used
        }
    }

    pub fn step(&mut self, ram: &mut MirrorVec<u8>) {
        trace!("DSP Step Begin");

        self.noise.step();

        let mut dsp_out = (0, 0);
        let mut echo_in = (0, 0);

        for voice in &mut self.voices {
            let sample = voice.step(&self.dir, ram, self.noise.level(), self.poll_key_state);

            dsp_out.0 = clamp16(dsp_out.0 + sample.0);
            dsp_out.1 = clamp16(dsp_out.1 + sample.1);

            if voice.echo_enabled() {
                echo_in.0 = clamp16(echo_in.0 + sample.0);
                echo_in.1 = clamp16(echo_in.1 + sample.1);
            }
        }

        dsp_out.0 = clamp16((dsp_out.0 * self.volume_left) >> 7);
        dsp_out.1 = clamp16((dsp_out.1 * self.volume_right) >> 7);

        let echo_out = self.echo.step(ram, echo_in);
        dsp_out.0 = clamp16(dsp_out.0 + echo_out.0);
        dsp_out.1 = clamp16(dsp_out.1 + echo_out.1);

        // TODO: Mute

        self.audio_queue
            .push_back((!dsp_out.0 as f32 / 32768.0, !dsp_out.1 as f32 / 32768.0));

        self.poll_key_state = !self.poll_key_state;

        trace!("DSP Step End");
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
    if value < i16::MIN as i32 || value > i16::MAX as i32 {
        warn!("Value clamped in mixer: {}", value);
    }

    value.clamp(i16::MIN as i32, i16::MAX as i32)
}

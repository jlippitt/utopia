use tracing::debug;
use voice::Voice;

mod voice;

const TOTAL_REGISTERS: usize = 128;

pub struct Dsp {
    address: u8,
    voices: [Voice; 8],
    data: [u8; TOTAL_REGISTERS],
}

impl Dsp {
    pub fn new() -> Self {
        Self {
            address: 0,
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
            data: [0; TOTAL_REGISTERS],
        }
    }

    pub fn address(&self) -> u8 {
        self.address
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
                    0x0c => (), // TODO: Main volume (left)
                    0x1c => (), // TODO: Main volume (right)
                    0x2c => (), // TODO: Echo volume (left)
                    0x3c => (), // TODO: Echo volume (right)
                    0x4c => (), // TODO: Key on
                    0x5c => (), // TODO: Key off
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
                    0x5d => (), // TODO: Directory
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

    fn voice(&self, address: u8) -> &Voice {
        &self.voices[(address >> 4) as usize]
    }

    fn voice_mut(&mut self, address: u8) -> &mut Voice {
        &mut self.voices[(address >> 4) as usize]
    }
}

use tracing::debug;

const TOTAL_REGISTERS: usize = 128;

pub struct Dsp {
    address: u8,
    data: [u8; TOTAL_REGISTERS],
}

impl Dsp {
    pub fn new() -> Self {
        Self {
            address: 0,
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
        self.data[self.address as usize & 0x7f]
    }

    pub fn write(&mut self, value: u8) {
        if self.address > 0x7f {
            return;
        }

        self.data[self.address as usize] = value;
        debug!("DSP Write: {:02X} <= {:02X}", self.address, value);
    }
}

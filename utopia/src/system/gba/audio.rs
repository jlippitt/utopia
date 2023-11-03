use crate::util::memory::{Masked, Reader, Writer};
use tracing::{trace, warn};

pub struct Audio {
    bias: u16,
}

impl Audio {
    pub fn new() -> Self {
        Self { bias: 0x0200 }
    }
}

impl Reader for Audio {
    type Value = u16;

    fn read_register(&self, address: u32) -> u16 {
        match address & 0xff {
            0x88 => self.bias,
            address => panic!("Unmapped Audio Read: {:02X}", address),
        }
    }
}

impl Writer for Audio {
    fn write_register(&mut self, address: u32, value: Masked<u16>) {
        match address & 0xff {
            0x88 => {
                self.bias = value.apply(self.bias) & 0xc3fe;
                trace!("Audio Bias: {:04X}", self.bias);
            }
            address => warn!(
                "Unmapped Audio Write: {:02X} <= {:04X}",
                address,
                value.get()
            ),
        }
    }
}

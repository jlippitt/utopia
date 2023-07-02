use super::{Mappings, Mbc};
use tracing::debug;

pub struct Mbc1 {
    ram_enable: bool,
    register: [u8; 2],
    mode: bool,
}

impl Mbc1 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            register: [0; 2],
            mode: false,
        }
    }

    fn update_mappings(&self, mappings: &mut Mappings) {
        if self.mode {
            panic!("MBC1 advanced banking mode not yet implemented");
        } else {
            mappings.rom[0] = 0;
        }

        mappings.rom[1] = ((self.register[1] as usize) << 19) | ((self.register[0] as usize) << 14);

        debug!("MBC1 ROM Mapping: {:?}", mappings.rom);
    }
}

impl Mbc for Mbc1 {
    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xe000 {
            0x0000 => {
                self.ram_enable = (value & 0x0f) == 0x0a;
                debug!("MBC1 RAM Enable: {}", self.ram_enable);
            }
            0x2000 => {
                self.register[0] = value & 0x1f;

                // Value of 0 behaves as if it was set to 1
                if self.register[0] == 0 {
                    self.register[0] = 1;
                }

                debug!("MBC1 Register 0: {:02X}", self.register[0]);
            }
            0x4000 => {
                self.register[1] = value & 0x03;
                debug!("MBC1 Register 1: {:02X}", self.register[1]);
            }
            0x6000 => {
                self.mode = (value & 0x01) != 0;
                debug!("MBC1 Mode: {}", self.mode);
            }
            _ => unreachable!(),
        }

        self.update_mappings(mappings);
    }
}

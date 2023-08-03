use super::{Mappings, Mbc};
use tracing::debug;

pub struct Mbc3 {
    rom_bank: u8,
    ram_bank: u8,
    ram_enable: bool,
}

impl Mbc3 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank: 1,
            ram_bank: 0,
        }
    }

    fn update_mappings(&self, mappings: &mut Mappings) {
        mappings.rom[1] = Mappings::ROM_PAGE_SIZE * (self.rom_bank as usize);

        mappings.ram = if self.ram_enable {
            Some(Mappings::RAM_PAGE_SIZE * (self.ram_bank as usize))
        } else {
            None
        };
    }
}

impl Mbc for Mbc3 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.rom[0] = 0;
        self.update_mappings(mappings)
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xe000 {
            0x0000 => {
                self.ram_enable = (value & 0x0f) == 0x0a;
                debug!("MBC3 RAM Enable: {}", self.ram_enable);
            }
            0x2000 => {
                self.rom_bank = value & 0x7f;

                // Value of 0 behaves as if it was set to 1
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }

                debug!("MBC3 ROM Bank: {}", self.rom_bank);
            }
            0x4000 => {
                if (value & 0x0f) >= 0x08 {
                    todo!("Real-Time Clock");
                }

                self.ram_bank = value & 0x03;
                debug!("MBC3 RAM Bank: {}", self.ram_bank);
            }
            0x6000 => {
                // TODO: Real-Time Clock
            }
            _ => unreachable!(),
        }

        self.update_mappings(mappings);
    }
}

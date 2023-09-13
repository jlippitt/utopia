use super::{Mappings, Mbc};
use tracing::debug;

pub struct Mbc5 {
    ram_enable: bool,
    rom_bank: u16,
    ram_bank: u8,
}

impl Mbc5 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank: 0,
            ram_bank: 0,
        }
    }

    fn update_mappings(&self, mappings: &mut Mappings) {
        mappings.ram = self
            .ram_enable
            .then_some(Mappings::RAM_PAGE_SIZE * self.ram_bank as usize);

        mappings.rom[0] = 0;
        mappings.rom[1] = Mappings::ROM_PAGE_SIZE * self.rom_bank as usize;

        debug!("MBC5 ROM Mapping: {:?}", mappings.rom);
        debug!("MBC5 RAM Mapping: {:?}", mappings.ram);
    }
}

impl Mbc for Mbc5 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        self.update_mappings(mappings)
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xe000 {
            0x0000 => {
                self.ram_enable = (value & 0x0f) == 0x0a;
                debug!("MBC5 RAM Enable: {}", self.ram_enable);
            }
            0x2000 => {
                self.rom_bank = (self.rom_bank & 0xff00) | value as u16;
                debug!("MBC5 ROM Bank: {:02X}", self.rom_bank);
            }
            0x4000 => {
                self.rom_bank = (self.rom_bank & 0xff) | ((value as u16 & 0x01) << 8);
                debug!("MBC5 ROM Bank: {:02X}", self.rom_bank);
            }
            0x6000 => {
                self.ram_bank = value & 0x0f;
                debug!("MBC5 RAM Bank: {}", self.ram_bank);
            }
            _ => unreachable!(),
        }

        self.update_mappings(mappings);
    }
}
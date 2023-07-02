use crate::util::MirrorVec;
use mbc::{Mappings, Mbc, MbcType};
use tracing::info;

mod mbc;

const BASE_ROM_SIZE: usize = 32768;

pub struct Cartridge {
    rom: MirrorVec<u8>,
    mappings: Mappings,
    mapper: MbcType,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let mapper_number = rom[0x0147];

        let rom_size = BASE_ROM_SIZE << rom[0x0148];

        let ram_size = match rom[0x0149] {
            2 => 8192,
            3 => 32768,
            4 => 131072,
            5 => 65536,
            _ => 0,
        };

        info!("Title: {}", String::from_utf8_lossy(&rom[0x0134..=0x0143]));
        info!("Mapper Number: {}", mapper_number);
        info!("ROM Size: {}", rom_size);
        info!("RAM Size: {}", ram_size);

        Self {
            rom: rom.into(),
            mappings: Mappings::new(),
            mapper: MbcType::new(mapper_number),
        }
    }

    pub fn read_rom(&self, index: usize) -> u8 {
        let offset = self.mappings.rom[(index >> 14) & 1];
        self.rom[offset | (index & 0x3fff)]
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        self.mapper
            .write_register(&mut self.mappings, address, value);
    }
}

use crate::util::MirrorVec;
use mbc::{Mappings, Mbc, MbcType};
use tracing::info;

mod mbc;

const BASE_ROM_SIZE: usize = 32768;

pub struct Cartridge {
    rom: MirrorVec<u8>,
    ram: MirrorVec<u8>,
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
            ram: MirrorVec::new(ram_size),
            mappings: Mappings::new(),
            mapper: MbcType::new(mapper_number),
        }
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        let offset = self.mappings.rom[(address as usize >> 14) & 1];
        self.rom[offset | (address as usize & 0x3fff)]
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if let Some(offset) = self.mappings.ram {
            self.ram[offset | (address as usize & 0x1fff)]
        } else {
            0xff
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        if let Some(offset) = self.mappings.ram {
            self.ram[offset | (address as usize & 0x1fff)] = value;
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        self.mapper
            .write_register(&mut self.mappings, address, value);
    }
}

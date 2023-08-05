use crate::util::mirror::{Mirror, MirrorVec};
use crate::{Mapped, MemoryMapper};
use mbc::{Mappings, Mbc, MbcType};
use std::error::Error;
use std::path::Path;
use tracing::info;

mod mbc;

const BASE_ROM_SIZE: usize = 32768;

const BATTERY_BACKED: [u8; 9] = [0x03, 0x06, 0x0d, 0x10, 0x13, 0x1b, 0x1e, 0x22, 0xff];

pub struct Cartridge<T: Mapped> {
    rom: MirrorVec<u8>,
    ram: Mirror<T>,
    mappings: Mappings,
    mapper: MbcType,
}

impl<T: Mapped> Cartridge<T> {
    pub fn new<U: MemoryMapper<Mapped = T>>(
        rom: Vec<u8>,
        rom_path: &Path,
        memory_mapper: &U,
    ) -> Result<Self, Box<dyn Error>> {
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
        info!("Mapper Number: {:02X}", mapper_number);
        info!("ROM Size: {}", rom_size);
        info!("RAM Size: {}", ram_size);

        let mut mappings = Mappings::new();
        let mut mapper = MbcType::new(mapper_number);
        mapper.init_mappings(&mut mappings);

        let battery_backed = ram_size > 0 && BATTERY_BACKED.contains(&mapper_number);
        let save_path = battery_backed.then(|| rom_path.with_extension("sav"));

        Ok(Self {
            rom: rom.into(),
            ram: memory_mapper.open(save_path.as_deref(), ram_size)?.into(),
            mappings,
            mapper,
        })
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

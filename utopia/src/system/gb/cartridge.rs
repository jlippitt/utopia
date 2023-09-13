use crate::util::mirror::{Mirror, MirrorVec};
use crate::{Mapped, MemoryMapper};
use mbc::{Mappings, Mbc, MbcType, RamMapping};
use std::error::Error;
use tracing::info;

mod mbc;

const BASE_ROM_SIZE: usize = 32768;

const BATTERY_BACKED: [u8; 9] = [0x03, 0x06, 0x0d, 0x10, 0x13, 0x1b, 0x1e, 0x22, 0xff];

pub struct Cartridge<T: Mapped> {
    rom: MirrorVec<u8>,
    ram: Mirror<T>,
    is_cgb: bool,
    mappings: Mappings,
    mapper: MbcType,
}

impl<T: Mapped> Cartridge<T> {
    pub fn new(
        rom: Vec<u8>,
        memory_mapper: &impl MemoryMapper<Mapped = T>,
    ) -> Result<Self, Box<dyn Error>> {
        let is_cgb = (rom[0x0143] & 0x80) != 0;
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
        info!("Model: {}", if is_cgb { "CGB" } else { "DMG" });
        info!("Mapper Number: {:02X}", mapper_number);
        info!("ROM Size: {}", rom_size);
        info!("RAM Size: {}", ram_size);

        let mut mappings = Mappings::new();
        let mut mapper = MbcType::new(mapper_number);
        mapper.init_mappings(&mut mappings);

        let battery_backed = ram_size > 0 && BATTERY_BACKED.contains(&mapper_number);
        info!("Battery Backed: {}", battery_backed);

        Ok(Self {
            rom: rom.into(),
            ram: memory_mapper.open(ram_size, battery_backed)?.into(),
            is_cgb,
            mappings,
            mapper,
        })
    }

    pub fn is_cgb(&self) -> bool {
        self.is_cgb
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        let offset = self.mappings.rom[(address as usize >> 14) & 1];
        self.rom[offset | (address as usize & 0x3fff)]
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        match self.mappings.ram {
            RamMapping::Offset(offset) => self.ram[offset | (address as usize & 0x1fff)],
            RamMapping::Custom => self.mapper.read_ram(address),
            RamMapping::None => 0xff,
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        match self.mappings.ram {
            RamMapping::Offset(offset) => self.ram[offset | (address as usize & 0x1fff)] = value,
            RamMapping::Custom => self.mapper.write_ram(address, value),
            RamMapping::None => (),
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        self.mapper
            .write_register(&mut self.mappings, address, value);
    }
}

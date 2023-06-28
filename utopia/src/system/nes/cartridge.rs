use crate::{
    system::nes::cartridge::mapper::{Mapper, MirrorMode},
    util::MirrorVec,
};
use mapper::{MapperType, Mappings, NameTable, PrgRead, PrgWrite};
use tracing::debug;

mod mapper;

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_MULTIPLIER: usize = 16384;
const CHR_ROM_MULTIPLIER: usize = 8192;
const CI_RAM_SIZE: usize = 2048;

pub struct Cartridge {
    prg_rom: MirrorVec<u8>,
    chr_data: MirrorVec<u8>,
    ci_ram: MirrorVec<u8>,
    mappings: Mappings,
    mapper: MapperType,
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Cartridge {
        let prg_rom_size = PRG_ROM_MULTIPLIER * (data[4] as usize);
        let chr_rom_size = CHR_ROM_MULTIPLIER * (data[5] as usize);
        let mirror_mode = if (data[6] & 0x01) != 0 {
            MirrorMode::Vertical
        } else {
            MirrorMode::Horizontal
        };
        let mapper_number = ((data[6] & 0xf0) >> 4) | (data[7] & 0xf0);

        debug!("PRG ROM Size: {}", prg_rom_size);
        debug!("CHR ROM Size: {}", chr_rom_size);
        debug!("Mirror Mode: {:?}", mirror_mode);
        debug!("Mapper Number: {}", mapper_number);

        let trainer_present = (data[6] & 0x04) != 0;
        let prg_rom_start = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let chr_rom_end = chr_rom_start + chr_rom_size;

        let prg_rom = Vec::from(&data[prg_rom_start..chr_rom_start]);
        let chr_data = Vec::from(&data[chr_rom_start..chr_rom_end]);

        let mut mappings = Mappings::new(mirror_mode);
        let mut mapper = MapperType::new(mapper_number, prg_rom_size);
        mapper.init_mappings(&mut mappings);

        Self {
            prg_rom: prg_rom.into(),
            chr_data: chr_data.into(),
            ci_ram: MirrorVec::new(CI_RAM_SIZE),
            mappings,
            mapper,
        }
    }

    pub fn read_prg(&self, address: u16, prev_value: u8) -> u8 {
        match self.mappings.prg_read[address as usize >> 12] {
            PrgRead::Rom(offset) => self.prg_rom[offset as usize | (address as usize & 0x0fff)],
            //PrgRead::Ram(_) => panic!("PRG RAM reads not yet implemented"),
            PrgRead::None => prev_value,
        }
    }

    pub fn write_prg(&mut self, address: u16, value: u8) {
        match self.mappings.prg_write[address as usize >> 12] {
            PrgWrite::Register => self
                .mapper
                .write_register(&mut self.mappings, address, value),
            //PrgWrite::Ram(_) => panic!("PRG RAM writes not yet implemented"),
            PrgWrite::None => (),
        }
    }

    pub fn read_name(&self, address: u16) -> u8 {
        let index = address as usize & 0x0fff;

        match self.mappings.name[index >> 10] {
            NameTable::Low => self.ci_ram[index & 0x03ff],
            NameTable::High => self.ci_ram[0x0400 | (index & 0x03ff)],
        }
    }

    pub fn write_name(&mut self, address: u16, value: u8) {
        let index = address as usize & 0x0fff;

        match self.mappings.name[index >> 10] {
            NameTable::Low => self.ci_ram[index & 0x03ff] = value,
            NameTable::High => self.ci_ram[0x0400 | (index & 0x03ff)] = value,
        }
    }

    pub fn read_chr(&self, address: u16) -> u8 {
        self.chr_data[address as usize]
    }

    pub fn write_chr(&mut self, _address: u16, _value: u8) {
        panic!("CHR RAM writes not yet implemented");
    }
}

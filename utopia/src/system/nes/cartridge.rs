use crate::util::MirrorVec;
use tracing::debug;

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_MULTIPLIER: usize = 16384;
const CHR_ROM_MULTIPLIER: usize = 8192;
const CI_RAM_SIZE: usize = 2048;

#[derive(Clone, Copy, Debug)]
enum PrgRead {
    Rom(u32),
    Ram(u32),
    None,
}

#[derive(Clone, Copy, Debug)]
enum PrgWrite {
    Register,
    Ram(u32),
    None,
}

#[derive(Clone, Copy, Debug)]
enum NameTable {
    Low,
    High,
}

struct Mappings {
    prg_read: [PrgRead; 16],
    prg_write: [PrgWrite; 16],
    name: [NameTable; 4],
}

pub struct Cartridge {
    prg_rom: MirrorVec<u8>,
    chr_data: MirrorVec<u8>,
    ci_ram: MirrorVec<u8>,
    mappings: Mappings,
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Cartridge {
        let prg_rom_size = PRG_ROM_MULTIPLIER * (data[4] as usize);
        let chr_rom_size = CHR_ROM_MULTIPLIER * (data[5] as usize);

        debug!("PRG ROM Size: {}", prg_rom_size);
        debug!("CHR ROM Size: {}", chr_rom_size);

        let trainer_present = (data[6] & 0x04) != 0;
        let prg_rom_start = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let chr_rom_end = chr_rom_start + chr_rom_size;

        let prg_rom = Vec::from(&data[prg_rom_start..chr_rom_start]);
        let chr_data = Vec::from(&data[chr_rom_start..chr_rom_end]);

        let name_tables = if (data[6] & 0x01) != 0 {
            debug!("Mirror Mode: Vertical");
            [
                NameTable::Low,
                NameTable::High,
                NameTable::Low,
                NameTable::High,
            ]
        } else {
            debug!("Mirror Mode: Horizontal");
            [
                NameTable::Low,
                NameTable::Low,
                NameTable::High,
                NameTable::High,
            ]
        };

        Self {
            prg_rom: prg_rom.into(),
            chr_data: chr_data.into(),
            ci_ram: MirrorVec::new(CI_RAM_SIZE),
            mappings: Mappings {
                prg_read: [
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::None,
                    PrgRead::Rom(0),
                    PrgRead::Rom(4096),
                    PrgRead::Rom(8192),
                    PrgRead::Rom(12288),
                    PrgRead::Rom(16384),
                    PrgRead::Rom(20480),
                    PrgRead::Rom(24576),
                    PrgRead::Rom(28672),
                ],
                prg_write: [PrgWrite::None; 16],
                name: name_tables,
            },
        }
    }

    pub fn read_prg(&self, address: u16, prev_value: u8) -> u8 {
        match self.mappings.prg_read[address as usize >> 12] {
            PrgRead::Rom(offset) => self.prg_rom[offset as usize | (address as usize & 0x0fff)],
            PrgRead::Ram(_) => panic!("PRG RAM reads not yet implemented"),
            PrgRead::None => prev_value,
        }
    }

    pub fn write_prg(&self, address: u16, _value: u8) {
        match self.mappings.prg_write[address as usize >> 12] {
            PrgWrite::Register => panic!("Mapper register writes not yet implemented"),
            PrgWrite::Ram(_) => panic!("PRG RAM writes not yet implemented"),
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

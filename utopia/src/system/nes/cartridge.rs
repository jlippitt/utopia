use crate::util::MirrorVec;
use tracing::{debug, warn};

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_MULTIPLIER: usize = 16384;
const CHR_ROM_MULTIPLIER: usize = 8192;
const CI_RAM_SIZE: usize = 2048;

#[derive(Clone, Copy, Debug)]
enum NameTable {
    Low,
    High,
}

struct Mappings {
    name_tables: [NameTable; 4],
}

pub struct Cartridge {
    prg_rom: MirrorVec<u8>,
    _chr_data: MirrorVec<u8>,
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
            [NameTable::Low, NameTable::High, NameTable::Low, NameTable::High]
        } else {
            debug!("Mirror Mode: Horizontal");
            [NameTable::Low, NameTable::Low, NameTable::High, NameTable::High]
        };

        Self {
            prg_rom: prg_rom.into(),
            _chr_data: chr_data.into(),
            ci_ram: MirrorVec::new(CI_RAM_SIZE),
            mappings: Mappings {
                name_tables,
            }
        }
    }

    pub fn read_prg_rom(&self, address: u16) -> u8 {
        self.prg_rom[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        if address >= 0x2000 {
            let index = address as usize & 0x0fff;

            match self.mappings.name_tables[index >> 10] {
                NameTable::Low => self.ci_ram[index & 0x03ff] = value,
                NameTable::High => self.ci_ram[0x0400 | (index & 0x03ff)] = value,
            }
        } else {
            warn!("CHR RAM writes not yet implemented");
        }
    }
}
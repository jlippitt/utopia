use super::{Interrupt, Mapper, Mappings, NameTable};
use tracing::{debug, warn};

const PRG_BANK_SIZE: usize = 8192;

pub struct Mmc5 {
    prg_mode: u8,
    prg_bank: [u8; 5],
    _prg_rom_size: usize,
    _interrupt: Interrupt,
}

impl Mmc5 {
    pub fn new(prg_rom_size: usize, interrupt: Interrupt) -> Self {
        Self {
            prg_mode: 3,
            prg_bank: [0, 0, 0, 0, 0xff],
            _prg_rom_size: prg_rom_size,
            _interrupt: interrupt,
        }
    }

    fn update_prg_mappings(&mut self, mappings: &mut Mappings) {
        match self.prg_mode {
            0 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 8, self.prg_bank[4] | 0x80 & 0xfc);
            }
            1 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 4, self.prg_bank[2] & 0xfe);
                map_prg(mappings, 12, 4, self.prg_bank[4] | 0x80 & 0xfe);
            }
            2 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 4, self.prg_bank[2] & 0xfe);
                map_prg(mappings, 12, 2, self.prg_bank[3]);
                map_prg(mappings, 14, 2, self.prg_bank[4] | 0x80);
            }
            3 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 2, self.prg_bank[1]);
                map_prg(mappings, 10, 2, self.prg_bank[2]);
                map_prg(mappings, 12, 2, self.prg_bank[3]);
                map_prg(mappings, 14, 2, self.prg_bank[4] | 0x80);
            }
            _ => unreachable!(),
        }

        debug!("MMC5 PRG Read Mappings: {:?}", mappings.prg_read);
        debug!("MMC5 PRG Write Mappings: {:?}", mappings.prg_write);
    }
}

impl Mapper for Mmc5 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers_with_read(5, 1);
        self.update_prg_mappings(mappings);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address {
            0x5000..=0x5015 => (), // TODO: MMC5 Audio
            0x5100 => {
                self.prg_mode = value & 0x03;
                debug!("MMC5 PRG Mode: {}", self.prg_mode);
                self.update_prg_mappings(mappings);
            }
            0x5101 => (), // TODO: CHR Mapping
            0x5104 => (), // TODO: ERAM
            0x5105 => {
                map_name(mappings, 0, value & 0x03);
                map_name(mappings, 1, (value >> 2) & 0x03);
                map_name(mappings, 2, (value >> 4) & 0x03);
                map_name(mappings, 3, value >> 6);
                debug!("MMC5 Name Mappings: {:?}", mappings.name);
            }
            0x5106 => (), // TODO: Fill Mode
            0x5107 => (), // TODO: Fill Mode
            0x5113..=0x5117 => {
                let index = (address - 0x5113) as usize;
                self.prg_bank[index] = value;
                debug!("MMC5 PRG Bank {}: {:02X}", index, value);
                self.update_prg_mappings(mappings);
            }
            0x5120..=0x512b => (), // TODO: CHR Mapping
            0x5200 => (),          // TODO: Vertical Split
            0x5203 => (),          // TODO: Scanline IRQ
            0x5204 => (),          // TODO: Scanline IRQ
            _ => unimplemented!("MMC5 Register Write {:04X} <= {:02X}", address, value),
        }
    }
}

fn map_prg(mappings: &mut Mappings, start: usize, len: usize, bank: u8) {
    if (bank & 0x80) != 0 {
        mappings.map_prg_rom(start, len, PRG_BANK_SIZE * bank as usize);
        mappings.unmap_prg_write(start, len);
    } else {
        mappings.map_prg_ram(start, len, PRG_BANK_SIZE * bank as usize);
    }
}

fn map_name(mappings: &mut Mappings, index: usize, value: u8) {
    mappings.name[index] = match value {
        0 => NameTable::Low,
        1 => NameTable::High,
        2 => {
            warn!("ERAM NameTable not yet implemented");
            NameTable::Low
        }
        3 => {
            warn!("Fill Mode NameTable not yet implemented");
            NameTable::Low
        }
        _ => unimplemented!("Custom nametables"),
    }
}

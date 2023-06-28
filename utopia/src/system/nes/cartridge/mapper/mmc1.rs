use super::{Mapper, Mappings, NameTable, MIRROR_HORIZONTAL, MIRROR_VERTICAL};
use tracing::debug;

const PRG_BANK_SIZE: usize = 16384;
const CHR_BANK_SIZE: usize = 4096;

pub struct Mmc1 {
    shift: u8,
    mirror_mode: u8,
    prg_rom_mode: u8,
    chr_mode: bool,
    chr_bank: [u8; 2],
    prg_bank: u8,
    prg_ram_enabled: bool,
    prg_rom_size: usize,
}

impl Mmc1 {
    pub fn new(prg_rom_size: usize) -> Self {
        Self {
            shift: 0x10,
            mirror_mode: 0,
            prg_rom_mode: 3,
            chr_mode: false,
            chr_bank: [0; 2],
            prg_bank: 0,
            prg_ram_enabled: true,
            prg_rom_size,
        }
    }

    fn write_internal_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        debug!("MMC1 Register Write: {:04X} <= {:02X}", address, value);

        match address & 0xe000 {
            0x8000 => {
                self.mirror_mode = value & 0x03;
                self.prg_rom_mode = (value & 0x0c) >> 2;
                self.chr_mode = (value & 0x10) != 0;
                debug!("MMC1 Mirror Mode: {}", self.mirror_mode);
                debug!("MMC1 PRG ROM Mode: {}", self.prg_rom_mode);
                debug!("MMC1 CHR Mode: {}", self.chr_mode as u32);
            }
            0xa000 => {
                self.chr_bank[0] = value;
                debug!("MMC1 CHR Bank 0: {}", self.chr_bank[0]);
            }
            0xc000 => {
                self.chr_bank[1] = value;
                debug!("MMC1 CHR Bank 1: {}", self.chr_bank[1]);
            }
            0xe000 => {
                self.prg_bank = value & 0x0f;
                self.prg_ram_enabled = (value & 0x10) == 0;
                debug!("MMC1 PRG Bank: {}", self.prg_bank);
                debug!("MMC1 PRG RAM Enabled: {}", self.prg_ram_enabled);
            }
            _ => unreachable!(),
        }

        self.update_mappings(mappings);
    }

    fn update_mappings(&self, mappings: &mut Mappings) {
        match self.prg_rom_mode {
            0 | 1 => mappings.map_prg_rom(8, 8, PRG_BANK_SIZE * (self.prg_bank & 0x0e) as usize),
            2 => {
                mappings.map_prg_rom(8, 4, 0);
                mappings.map_prg_rom(12, 4, PRG_BANK_SIZE * self.prg_bank as usize);
            }
            3 => {
                mappings.map_prg_rom(8, 4, PRG_BANK_SIZE * self.prg_bank as usize);
                mappings.map_prg_rom(12, 4, self.prg_rom_size - PRG_BANK_SIZE);
            }
            _ => unreachable!(),
        }

        // TODO: PRG RAM mapping

        mappings.name = match self.mirror_mode {
            0 => [NameTable::Low; 4],
            1 => [NameTable::High; 4],
            2 => MIRROR_VERTICAL,
            3 => MIRROR_HORIZONTAL,
            _ => unreachable!(),
        };

        if self.chr_mode {
            mappings.map_chr(0, 4, CHR_BANK_SIZE * self.chr_bank[0] as usize);
            mappings.map_chr(4, 4, CHR_BANK_SIZE * self.chr_bank[1] as usize);
        } else {
            mappings.map_chr(0, 8, CHR_BANK_SIZE * 2 * (self.chr_bank[0] & 0x1e) as usize);
        }

        debug!("MMC1 PRG Read Mappings: {:?}", mappings.prg_read);
        debug!("MMC1 PRG Write Mappings: {:?}", mappings.prg_write);
        debug!("MMC1 Name Mappings: {:?}", mappings.name);
        debug!("MMC1 CHR Mappings: {:?}", mappings.chr);
    }
}

impl Mapper for Mmc1 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers(8, 8);
        self.update_mappings(mappings);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        // TODO: Detect and ignore writes on consecutive cycles
        if (value & 0x80) != 0 {
            self.shift = 0x10;
            self.prg_rom_mode = 3;
            debug!("MMC1 Shift: {:02X}", self.shift);
            debug!("MMC1 PRG ROM Mode: {}", self.prg_rom_mode);
            self.update_mappings(mappings);
            return;
        }

        let done = (self.shift & 0x01) != 0;

        self.shift = (self.shift >> 1) | ((value & 0x01) << 4);

        if done {
            self.write_internal_register(mappings, address, self.shift);
            self.shift = 0x10;
        }

        debug!("MMC1 Shift: {:02X}", self.shift);
    }
}

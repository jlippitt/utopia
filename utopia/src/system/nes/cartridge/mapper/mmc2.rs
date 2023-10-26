use super::{Mapper, Mappings, MirrorMode};
use tracing::trace;

const PRG_BANK_SIZE: usize = 8192;
const CHR_BANK_SIZE: usize = 4096;

pub struct Mmc2 {
    chr_bank: [[u8; 2]; 2],
    chr_latch: [bool; 2],
    prg_rom_size: usize,
}

impl Mmc2 {
    pub fn new(prg_rom_size: usize) -> Self {
        Self {
            chr_bank: [[0; 2]; 2],
            chr_latch: [false; 2],
            prg_rom_size,
        }
    }

    fn update_chr_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_chr(
            0,
            4,
            CHR_BANK_SIZE * self.chr_bank[0][self.chr_latch[0] as usize] as usize,
        );

        mappings.map_chr(
            4,
            4,
            CHR_BANK_SIZE * self.chr_bank[1][self.chr_latch[1] as usize] as usize,
        );

        trace!("MMC2 CHR Mapping: {:?}", mappings.chr);
    }
}

impl Mapper for Mmc2 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_prg_rom(8, 2, 0);
        mappings.map_prg_rom(10, 6, self.prg_rom_size - (PRG_BANK_SIZE * 3));
        mappings.map_registers(10, 6);
        trace!("MMC2 PRG Read Mapping: {:?}", mappings.prg_read);
        trace!("MMC2 PRG Write Mapping: {:?}", mappings.prg_write);
        self.update_chr_mappings(mappings);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xf000 {
            0xa000 => {
                mappings.map_prg_rom(8, 2, PRG_BANK_SIZE * (value as usize & 0x0f));
                trace!("MMC2 PRG Read Mapping: {:?}", mappings.prg_read);
            }
            0xb000 => {
                self.chr_bank[0][0] = value & 0x1f;
                trace!("MMC2 CHR Bank 0 (FD): {}", self.chr_bank[0][0]);
                self.update_chr_mappings(mappings);
            }
            0xc000 => {
                self.chr_bank[0][1] = value & 0x1f;
                trace!("MMC2 CHR Bank 0 (FE): {}", self.chr_bank[0][1]);
                self.update_chr_mappings(mappings);
            }
            0xd000 => {
                self.chr_bank[1][0] = value & 0x1f;
                trace!("MMC2 CHR Bank 1 (FD): {}", self.chr_bank[1][0]);
                self.update_chr_mappings(mappings);
            }
            0xe000 => {
                self.chr_bank[1][1] = value & 0x1f;
                trace!("MMC2 CHR Bank 1 (FE): {}", self.chr_bank[1][1]);
                self.update_chr_mappings(mappings);
            }
            0xf000 => {
                let mirror_mode = if (value & 0x01) != 0 {
                    MirrorMode::Horizontal
                } else {
                    MirrorMode::Vertical
                };

                trace!("MMC2 Mirror Mode: {:?}", mirror_mode);
                mappings.mirror_nametables(mirror_mode);
                trace!("MMC2 Name Mappings: {:?}", mappings.name);
            }
            _ => unreachable!(),
        }
    }

    fn on_ppu_chr_fetch(&mut self, mappings: &mut Mappings, ppu_address: u16) {
        match ppu_address {
            0x0fd8 => {
                self.chr_latch[0] = false;
                self.update_chr_mappings(mappings)
            }
            0x0fe8 => {
                self.chr_latch[0] = true;
                self.update_chr_mappings(mappings)
            }
            0x1fd8..=0x1fdf => {
                self.chr_latch[1] = false;
                self.update_chr_mappings(mappings)
            }
            0x1fe8..=0x1fef => {
                self.chr_latch[1] = true;
                self.update_chr_mappings(mappings)
            }
            _ => (),
        }
    }
}

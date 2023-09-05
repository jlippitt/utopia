use super::{Interrupt, Mapper, Mappings, CHR_PAGE_SIZE};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::debug;

const PRG_BANK_SIZE: usize = 8192;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum NameTable {
    Low,
    High,
    Eram,
    Fill,
}

pub struct Mmc5 {
    prg_mode: u8,
    prg_bank: [u8; 5],
    chr_mode: u8,
    chr_bank: [u8; 12],
    name_bank: [NameTable; 4],
    sprite_mapping: bool,
    render_enabled: bool,
    _prg_rom_size: usize,
    _interrupt: Interrupt,
}

impl Mmc5 {
    pub fn new(prg_rom_size: usize, interrupt: Interrupt) -> Self {
        Self {
            prg_mode: 3,
            prg_bank: [0, 0, 0, 0, 0xff],
            chr_mode: 0,
            chr_bank: [0; 12],
            name_bank: [NameTable::Low; 4],
            sprite_mapping: false,
            render_enabled: false,
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

    fn update_chr_mappings(&mut self, mappings: &mut Mappings) {
        match self.chr_mode {
            0 => {
                mappings.map_chr(0, 8, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
            }
            1 => {
                mappings.map_chr(0, 4, CHR_PAGE_SIZE * self.chr_bank[3] as usize);
                mappings.map_chr(4, 4, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
            }
            2 => {
                mappings.map_chr(0, 2, CHR_PAGE_SIZE * self.chr_bank[1] as usize);
                mappings.map_chr(2, 2, CHR_PAGE_SIZE * self.chr_bank[3] as usize);
                mappings.map_chr(4, 2, CHR_PAGE_SIZE * self.chr_bank[5] as usize);
                mappings.map_chr(6, 2, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
            }
            3 => {
                mappings.map_chr(0, 1, CHR_PAGE_SIZE * self.chr_bank[0] as usize);
                mappings.map_chr(1, 1, CHR_PAGE_SIZE * self.chr_bank[1] as usize);
                mappings.map_chr(2, 1, CHR_PAGE_SIZE * self.chr_bank[2] as usize);
                mappings.map_chr(3, 1, CHR_PAGE_SIZE * self.chr_bank[3] as usize);
                mappings.map_chr(4, 1, CHR_PAGE_SIZE * self.chr_bank[4] as usize);
                mappings.map_chr(5, 1, CHR_PAGE_SIZE * self.chr_bank[5] as usize);
                mappings.map_chr(6, 1, CHR_PAGE_SIZE * self.chr_bank[6] as usize);
                mappings.map_chr(7, 1, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
            }
            _ => unreachable!(),
        }

        // TODO: 8x16 Sprite Banks

        debug!("MMC5 CHR Mappings: {:?}", mappings.chr);
    }
}

impl Mapper for Mmc5 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers(2, 2);
        mappings.map_registers_with_read(5, 1);
        self.update_prg_mappings(mappings);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        if address <= 0x3fff {
            match address & 7 {
                0 => {
                    self.sprite_mapping = (value & 0x20) != 0;
                    debug!("MMC5 Sprite Bank Mapping: {}", self.sprite_mapping);
                }
                1 => {
                    self.render_enabled = (value & 0x18) != 0;
                    debug!("MMC5 Render Enabled: {}", self.render_enabled);
                }
                _ => (),
            }

            return;
        }

        match address {
            0x5000..=0x5015 => (), // TODO: MMC5 Audio
            0x5100 => {
                self.prg_mode = value & 0x03;
                debug!("MMC5 PRG Mode: {}", self.prg_mode);
                self.update_prg_mappings(mappings);
            }
            0x5101 => {
                self.chr_mode = value & 0x03;
                debug!("MMC5 CHR Mode: {}", self.chr_mode);
                self.update_chr_mappings(mappings);
            }
            0x5104 => (), // TODO: ERAM
            0x5105 => {
                self.name_bank[0] = NameTable::from_u8(value & 0x03).unwrap();
                self.name_bank[1] = NameTable::from_u8((value >> 2) & 0x03).unwrap();
                self.name_bank[2] = NameTable::from_u8((value >> 4) & 0x03).unwrap();
                self.name_bank[3] = NameTable::from_u8(value >> 6).unwrap();
                debug!("MMC5 Name Banks: {:?}", self.name_bank);
            }
            0x5106 => (), // TODO: Fill Mode
            0x5107 => (), // TODO: Fill Mode
            0x5113..=0x5117 => {
                let index = (address - 0x5113) as usize;
                self.prg_bank[index] = value;
                debug!("MMC5 PRG Bank {}: {:02X}", index, value);
                self.update_prg_mappings(mappings);
            }
            0x5120..=0x512b => {
                let index = (address - 0x5120) as usize;
                self.chr_bank[index] = value;
                debug!("MMC5 CHR Bank {}: {:02X}", index, value);
                self.update_chr_mappings(mappings);
            }
            0x5200 => (), // TODO: Vertical Split
            0x5203 => (), // TODO: Scanline IRQ
            0x5204 => (), // TODO: Scanline IRQ
            _ => unimplemented!("MMC5 Register Write {:04X} <= {:02X}", address, value),
        }
    }

    fn read_name(
        &mut self,
        _mappings: &mut Mappings,
        ci_ram: &crate::util::MirrorVec<u8>,
        address: u16,
    ) -> u8 {
        let index = address as usize & 0x0fff;

        match self.name_bank[index >> 10] {
            NameTable::Low => ci_ram[index & 0x03ff],
            NameTable::High => ci_ram[0x0400 | (index & 0x03ff)],
            NameTable::Eram => unimplemented!("ERAM Name Tables"),
            NameTable::Fill => unimplemented!("Fill Mode Name Tables"),
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

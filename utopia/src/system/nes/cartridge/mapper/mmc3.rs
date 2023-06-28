use super::{Mapper, Mappings, MirrorMode, CHR_PAGE_SIZE};
use tracing::debug;

const PRG_BANK_SIZE: usize = 8192;

pub struct Mmc3 {
    registers: [u8; 8],
    register_select: u8,
    prg_rom_mode: bool,
    chr_mode: bool,
    prg_rom_size: usize,
}

impl Mmc3 {
    pub fn new(prg_rom_size: usize) -> Self {
        Self {
            registers: [0; 8],
            register_select: 0,
            prg_rom_mode: false,
            chr_mode: false,
            prg_rom_size,
        }
    }

    fn update_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_prg_rom(
            self.prg(8),
            2,
            PRG_BANK_SIZE * (self.registers[6] & 0x3f) as usize,
        );

        mappings.map_prg_rom(10, 2, PRG_BANK_SIZE * (self.registers[7] & 0x3f) as usize);
        mappings.map_prg_rom(self.prg(12), 2, self.prg_rom_size - PRG_BANK_SIZE * 2);
        mappings.map_prg_rom(14, 2, self.prg_rom_size - PRG_BANK_SIZE);

        mappings.map_chr(
            self.chr(0),
            2,
            CHR_PAGE_SIZE * (self.registers[0] & 0xfe) as usize,
        );

        mappings.map_chr(
            self.chr(2),
            2,
            CHR_PAGE_SIZE * (self.registers[1] & 0xfe) as usize,
        );

        mappings.map_chr(self.chr(4), 1, CHR_PAGE_SIZE * self.registers[2] as usize);
        mappings.map_chr(self.chr(5), 1, CHR_PAGE_SIZE * self.registers[3] as usize);
        mappings.map_chr(self.chr(6), 1, CHR_PAGE_SIZE * self.registers[4] as usize);
        mappings.map_chr(self.chr(7), 1, CHR_PAGE_SIZE * self.registers[5] as usize);

        debug!("MMC3 PRG Read Mappings: {:?}", mappings.prg_read);
        debug!("MMC3 CHR Mappings: {:?}", mappings.chr);
    }

    fn prg(&self, index: usize) -> usize {
        index ^ ((self.prg_rom_mode as usize) << 2)
    }

    fn chr(&self, index: usize) -> usize {
        index ^ ((self.chr_mode as usize) << 2)
    }
}

impl Mapper for Mmc3 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers(8, 8);
        self.update_mappings(mappings);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xe001 {
            0x8000 => {
                self.register_select = value & 0x07;
                self.prg_rom_mode = (value & 0x40) != 0;
                self.chr_mode = (value & 0x80) != 0;
                debug!("MMC3 Register Select: {}", self.register_select);
                debug!("MMC3 PRG ROM Mode: {}", self.prg_rom_mode as u32);
                debug!("MMC3 CHR Mode: {}", self.chr_mode as u32);
                self.update_mappings(mappings);
            }
            0x8001 => {
                self.registers[self.register_select as usize] = value;
                debug!("MMC3 Register {}: {}", self.register_select, value);
                self.update_mappings(mappings);
            }
            0xa000 => {
                let mirror_mode = if (value & 0x01) != 0 {
                    MirrorMode::Horizontal
                } else {
                    MirrorMode::Vertical
                };

                debug!("MMC3 Mirror Mode: {:?}", mirror_mode);
                mappings.mirror_nametables(mirror_mode);
                debug!("MMC3 Name Mappings: {:?}", mappings.name);
            }
            0xa001 => {
                match value & 0xc0 {
                    0xc0 => {
                        mappings.map_prg_ram_read_only(6, 2, 0);
                        debug!("MMC3 PRG RAM Read-Only");
                    }
                    0x80 => {
                        mappings.map_prg_ram(6, 2, 0);
                        debug!("MMC3 PRG RAM Read/Write");
                    }
                    _ => {
                        mappings.unmap_prg(6, 2);
                        debug!("MMC3 PRG RAM Disabled");
                    }
                }

                debug!("MMC3 PRG Read Mappings: {:?}", mappings.prg_read);
                debug!("MMC3 PRG Write Mappings: {:?}", mappings.prg_write);
            }
            0xc000 => {
                // TODO: IRQ
            }
            0xc001 => {
                // TODO: IRQ
            }
            0xe000 => {
                // TODO: IRQ
            }
            0xe001 => {
                // TODO: IRQ
            }
            _ => unreachable!(),
        }
    }
}

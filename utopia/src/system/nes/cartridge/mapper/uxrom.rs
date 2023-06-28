use super::{Mapper, Mappings};
use tracing::debug;

const PRG_BANK_SIZE: usize = 16384;

pub struct UxRom {
    prg_rom_size: usize,
}

impl UxRom {
    pub fn new(prg_rom_size: usize) -> Self {
        Self { prg_rom_size }
    }
}

impl Mapper for UxRom {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_prg_rom(8, 4, 0);
        mappings.map_prg_rom(12, 4, self.prg_rom_size - PRG_BANK_SIZE);
        mappings.map_registers(8, 8);
        debug!("UxROM PRG Read Mapping: {:?}", mappings.prg_read);
        debug!("UxROM PRG Write Mapping: {:?}", mappings.prg_write);
    }

    fn write_register(&mut self, mappings: &mut Mappings, _address: u16, value: u8) {
        mappings.map_prg_rom(8, 4, PRG_BANK_SIZE * (value as usize & 0x0f));
        debug!("UxROM PRG Read Mapping: {:?}", mappings.prg_read);
    }
}

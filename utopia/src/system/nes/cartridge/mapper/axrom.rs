use super::{Mapper, Mappings, NameTable};
use tracing::trace;

const PRG_BANK_SIZE: usize = 32768;

pub struct AxRom;

impl AxRom {
    pub fn new() -> Self {
        Self
    }
}

impl Mapper for AxRom {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers(8, 8);
    }

    fn write_register(&mut self, mappings: &mut Mappings, _address: u16, value: u8) {
        mappings.map_prg_rom(8, 8, PRG_BANK_SIZE * (value & 0x07) as usize);
        trace!("AxROM PRG Read Mapping: {:?}", mappings.prg_read);

        mappings.name = if (value & 0x10) != 0 {
            [NameTable::HIGH; 4]
        } else {
            [NameTable::LOW; 4]
        };

        trace!("AxROM Name Mapping: {:?}", mappings.name);
    }
}

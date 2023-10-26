use super::{Mapper, Mappings};
use tracing::trace;

const CHR_BANK_SIZE: usize = 8192;

pub struct CnRom;

impl CnRom {
    pub fn new() -> Self {
        Self
    }
}

impl Mapper for CnRom {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers(8, 8);
    }

    fn write_register(&mut self, mappings: &mut Mappings, _address: u16, value: u8) {
        mappings.map_chr(0, 8, CHR_BANK_SIZE * value as usize);
        trace!("CNROM CHR Mapping: {:?}", mappings.chr);
    }
}

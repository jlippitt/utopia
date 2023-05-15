use super::rom::{self, ParsedRom};
use crate::core::mos6502::Bus;

pub struct Hardware {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let ParsedRom { prg_rom, chr } = rom::parse(rom_data);

        Hardware { prg_rom, chr }
    }
}

impl Bus for Hardware {
    fn read(&mut self, _address: u16) -> u8 {
        // TODO
        0
    }
}

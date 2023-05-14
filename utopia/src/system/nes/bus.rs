use super::rom::{self, ParsedRom};

pub struct Bus {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
}

impl Bus {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let ParsedRom { prg_rom, chr } = rom::parse(rom_data);

        Bus { prg_rom, chr }
    }
}

use super::rom::{self, ParsedRom};
use crate::core::mos6502::Bus;
use crate::util::MirrorVec;

const WRAM_SIZE: usize = 2048;

pub struct Hardware {
    prg_rom: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    chr: MirrorVec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let ParsedRom { prg_rom, chr } = rom::parse(rom_data);

        Hardware {
            prg_rom: prg_rom.into(),
            wram: MirrorVec::new(WRAM_SIZE),
            chr: chr.into(),
        }
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u16) -> u8 {
        match address >> 13 {
            0 => self.wram[address as usize],
            1 => panic!("PPU registers not yet implemented"),
            2 => panic!("2A03 registers not yet implemented"),
            3 => panic!("PRG RAM not yet implemented"),
            _ => self.prg_rom[address as usize],
        }
    }
}

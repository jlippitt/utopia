use super::ppu::Ppu;
use super::rom::{self, ParsedRom};
use crate::core::mos6502::Bus;
use crate::util::MirrorVec;
use std::fmt;
use tracing::warn;

const WRAM_SIZE: usize = 2048;

pub struct Hardware {
    cycles: u64,
    prg_rom: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    ppu: Ppu,
    chr: MirrorVec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let ParsedRom { prg_rom, chr } = rom::parse(rom_data);

        Hardware {
            cycles: 0,
            prg_rom: prg_rom.into(),
            wram: MirrorVec::new(WRAM_SIZE),
            ppu: Ppu::new(),
            chr: chr.into(),
        }
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u16) -> u8 {
        self.cycles += 12;
        self.ppu.step();
        self.ppu.step();
        self.ppu.step();

        match address >> 13 {
            0 => self.wram[address as usize],
            1 => self.ppu.read(address),
            2 => panic!("2A03 register reads not yet implemented"),
            3 => panic!("PRG RAM reads not yet implemented"),
            _ => self.prg_rom[address as usize],
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cycles += 4;
        self.ppu.step();

        match address >> 13 {
            0 => self.wram[address as usize] = value,
            1 => self.ppu.write(address, value),
            2 => warn!("2A03 register writes not yet implemented"),
            3 => panic!("PRG RAM writes not yet implemented"),
            _ => panic!("Mapper register writes not yet implemented"),
        };

        self.cycles += 8;
        self.ppu.step();
        self.ppu.step();
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "V={} T={}", self.ppu.v_counter(), self.cycles)
    }
}

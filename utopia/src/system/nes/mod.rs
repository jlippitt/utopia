use crate::core::mos6502::{Bus, Core, Interrupt};
use crate::util::MirrorVec;
use super::System;
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};
use cartridge::Cartridge;
use ppu::Ppu;

const WRAM_SIZE: usize = 2048;

mod cartridge;
mod ppu;

pub fn create(rom_data: Vec<u8>) -> Result<Box<dyn System>, Box<dyn Error>> {
    let hw = Hardware::new(rom_data);
    let core = Core::new(hw);

    Ok(Box::new(NES { core }))
}

pub struct NES {
    core: Core<Hardware>,
}

impl System for NES {
    fn width(&self) -> u32 { 240 }

    fn height(&self) -> u32 { 224 }

    fn run(&mut self) {
        loop {
            self.core.step();
            debug!("{}", self.core);
        }
    }
}

struct Hardware {
    cycles: u64,
    interrupt: Interrupt,
    cartridge: Cartridge,
    wram: MirrorVec<u8>,
    ppu: Ppu,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            cycles: 0,
            interrupt: 0,
            cartridge: Cartridge::new(rom_data),
            wram: MirrorVec::new(WRAM_SIZE),
            ppu: Ppu::new(),
        }
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u16) -> u8 {
        self.cycles += 12;
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);

        match address >> 13 {
            0 => self.wram[address as usize],
            1 => self.ppu.read(&mut self.cartridge, &mut self.interrupt, address),
            2 => match address {
                0x4016..=0x4017 => 0, // TODO: Joypad ports
                0x4000..=0x401f => 0, // TODO: APU ports
                _ => panic!("Read from unmapped address"),
            },
            3 => {
                //panic!("PRG RAM reads not yet implemented"),
                0
            },
            _ => self.cartridge.read_prg_rom(address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cycles += 4;
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);

        match address >> 13 {
            0 => self.wram[address as usize] = value,
            1 => self.ppu.write(&mut self.cartridge, &mut self.interrupt, address, value),
            2 => warn!("2A03 register writes not yet implemented"),
            3 => {
                //panic!("PRG RAM writes not yet implemented"),
                if address >= 0x6004 {
                    print!("{}", value as char);
                }
            },
            _ => panic!("Mapper register writes not yet implemented"),
        };

        self.cycles += 8;
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);
    }

    fn poll(&mut self) -> Interrupt {
        self.interrupt
    }

    fn acknowledge(&mut self, interrupt: Interrupt) {
        self.interrupt &= !interrupt;
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "V={} T={}", self.ppu.v_counter(), self.cycles)
    }
}

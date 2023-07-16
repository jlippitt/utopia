use super::{JoypadState, System};
use crate::core::wdc65c816::{Bus, Core};
use crate::util::MirrorVec;
use apu::Apu;
use memory::{Page, TOTAL_PAGES};
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};

mod apu;
mod memory;

const WIDTH: usize = 512;
const HEIGHT: usize = 448;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

const WRAM_SIZE: usize = 131072;

const FAST_CYCLES: u64 = 6;
const SLOW_CYCLES: u64 = 8;
const EXTRA_SLOW_CYCLES: u64 = 12;

pub struct Snes {
    core: Core<Hardware>,
}

impl Snes {
    pub fn new(rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let hw = Hardware::new(rom_data);
        let core = Core::new(hw);
        Ok(Snes { core })
    }
}

impl System for Snes {
    fn width(&self) -> usize {
        WIDTH
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        &PIXELS
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        loop {
            self.core.step();
            debug!("{}", self.core);
        }
    }
}

pub struct Hardware {
    cycles: u64,
    mdr: u8,
    pages: [Page; TOTAL_PAGES],
    rom: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    apu: Apu,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let pages = memory::map();

        Self {
            cycles: 0,
            mdr: 0,
            pages,
            rom: rom_data.into(),
            wram: MirrorVec::new(WRAM_SIZE),
            apu: Apu::new(),
        }
    }

    fn read_bus_b(&mut self, address: u8) -> u8 {
        match address {
            0x00..=0x3f => todo!("PPU reads"),
            0x40..=0x7f => self.apu.read(address),
            0x80..=0x83 => todo!("WRAM registers"),
            _ => {
                warn!("Unmapped Bus B read: {:02X}", address);
                self.mdr
            }
        }
    }

    fn write_bus_b(&mut self, address: u8, value: u8) {
        match address {
            0x00..=0x3f => (), // TODO: PPU writes
            0x40..=0x7f => self.apu.write(address, value),
            0x80..=0x83 => todo!("WRAM registers"),
            _ => warn!("Unmapped Bus B write: {:02X} <= {:02X}", address, value),
        }
    }

    fn cycles_for_address(&self, address: u32) -> u64 {
        if (address & 0x408000) != 0 {
            return if (address & 0x800000) != 0 {
                todo!("FastROM");
            } else {
                SLOW_CYCLES
            };
        }

        if (address.wrapping_add(0x6000) & 0x4000) != 0 {
            return SLOW_CYCLES;
        }

        if (address.wrapping_sub(0x4000) & 0x7e00) != 0 {
            return FAST_CYCLES;
        }

        return EXTRA_SLOW_CYCLES;
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        self.cycles += FAST_CYCLES;
    }

    fn read(&mut self, address: u32) -> u8 {
        self.cycles += self.cycles_for_address(address) - 4;

        self.mdr = match self.pages[(address >> 13) as usize] {
            Page::Rom(offset) => self.rom[(offset | (address & 0x1fff)) as usize],
            Page::Wram(offset) => self.wram[(offset | (address & 0x1fff)) as usize],
            Page::ExternalRegisters => match address & 0x1f00 {
                0x0100 => self.read_bus_b(address as u8),
                _ => {
                    warn!("Unmapped external register read: {:06X}", address);
                    self.mdr
                }
            },
            Page::InternalRegisters => panic!("Internal register reads not yet implemented"),
            Page::OpenBus => {
                warn!("Unmapped Bus A read: {:06X}", address);
                self.mdr
            }
        };

        self.cycles += 4;

        self.mdr
    }

    fn write(&mut self, address: u32, value: u8) {
        self.cycles += self.cycles_for_address(address);
        self.mdr = value;

        match self.pages[(address >> 13) as usize] {
            Page::Rom(..) => warn!("Write to ROM area: {:06X} <= {:02X}", address, value),
            Page::Wram(offset) => self.wram[(offset | (address & 0x1fff)) as usize] = value,
            Page::ExternalRegisters => match address & 0x1f00 {
                0x0100 => self.write_bus_b(address as u8, value),
                _ => warn!(
                    "Unmapped external register write: {:06X} <= {:02X}",
                    address, value
                ),
            },
            Page::InternalRegisters => (), // TODO
            Page::OpenBus => warn!("Unmapped Bus A write: {:06X} <= {:02X}", address, value),
        }
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={}", self.cycles)
    }
}

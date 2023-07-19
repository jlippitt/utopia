use super::{BiosLoader, JoypadState, System};
use crate::core::wdc65c816::{Bus, Core, Interrupt, INT_NMI};
use crate::util::MirrorVec;
use apu::Apu;
use clock::{Clock, Event, FAST_CYCLES};
use memory::{Page, TOTAL_PAGES};
use registers::Registers;
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};

mod apu;
mod clock;
mod memory;
mod registers;

const WIDTH: usize = 512;
const HEIGHT: usize = 448;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

const WRAM_SIZE: usize = 131072;

// TODO: Overscan
const VBLANK_LINE: u32 = 225;

pub struct Snes {
    core: Core<Hardware>,
}

impl Snes {
    pub fn new(rom_data: Vec<u8>, bios_loader: &impl BiosLoader) -> Result<Self, Box<dyn Error>> {
        let ipl_rom = bios_loader.load("ipl_rom")?;
        let hw = Hardware::new(rom_data, ipl_rom);
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
    clock: Clock,
    mdr: u8,
    interrupt: Interrupt,
    pages: [Page; TOTAL_PAGES],
    rom: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    regs: Registers,
    apu: Apu,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>, ipl_rom: Vec<u8>) -> Self {
        let pages = memory::map();

        Self {
            clock: Clock::new(),
            mdr: 0,
            interrupt: 0,
            pages,
            rom: rom_data.into(),
            wram: MirrorVec::new(WRAM_SIZE),
            regs: Registers::new(),
            apu: Apu::new(ipl_rom),
        }
    }

    fn step(&mut self, cycles: u64) {
        self.clock.add_cycles(cycles);

        while let Some(event) = self.clock.event() {
            match event {
                Event::NewLine => {
                    let line = self.clock.line();

                    if line == VBLANK_LINE {
                        self.regs.set_nmi_occurred(true);

                        if self.regs.nmi_raised() {
                            self.interrupt |= INT_NMI;
                        }
                    } else if line == 0 {
                        self.regs.set_nmi_occurred(false);
                        self.interrupt &= !INT_NMI;
                    }
                }
            }
        }
    }

    fn read_bus_b(&mut self, address: u8) -> u8 {
        match address {
            0x00..=0x3f => todo!("PPU reads"),
            0x40..=0x7f => {
                self.apu.run_until(self.clock.cycles());
                self.apu.read(address)
            }
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
            0x40..=0x7f => {
                self.apu.run_until(self.clock.cycles());
                self.apu.write(address, value);
            }
            0x80..=0x83 => todo!("WRAM registers"),
            _ => warn!("Unmapped Bus B write: {:02X} <= {:02X}", address, value),
        }
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        self.step(FAST_CYCLES);
    }

    fn read(&mut self, address: u32) -> u8 {
        self.step(self.clock.cycles_for_address(address) - 4);

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
            Page::InternalRegisters => match address & 0x1f00 {
                0x0000 => 0, // TODO: NES-style joypads
                0x0200 => self.read_register(address as u8, self.mdr),
                0x0300 => todo!("DMA reads"),
                _ => {
                    warn!("Unmapped internal register read: {:06X}", address);
                    self.mdr
                }
            },
            Page::OpenBus => {
                warn!("Unmapped Bus A read: {:06X}", address);
                self.mdr
            }
        };

        self.step(4);

        self.mdr
    }

    fn write(&mut self, address: u32, value: u8) {
        self.step(self.clock.cycles_for_address(address));
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
            Page::InternalRegisters => match address & 0x1f00 {
                0x0000 => (), // TODO: NES-style joypads
                0x0200 => self.write_register(address as u8, value),
                0x0300 => (), // TODO: DMA
                _ => warn!(
                    "Unmapped external register write: {:06X} <= {:02X}",
                    address, value
                ),
            },
            Page::OpenBus => warn!("Unmapped Bus A write: {:06X} <= {:02X}", address, value),
        }
    }

    fn poll(&self) -> Interrupt {
        self.interrupt
    }

    fn acknowledge(&mut self, interrupt: Interrupt) {
        self.interrupt &= !interrupt;
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.clock)
    }
}

use super::{BiosLoader, JoypadState, System};
use crate::core::wdc65c816::{Bus, Core, Interrupt, INT_NMI};
use crate::util::MirrorVec;
use apu::Apu;
use clock::{Clock, Event, FAST_CYCLES};
use dma::Dma;
use memory::{Page, TOTAL_PAGES};
use ppu::{Ppu, HEIGHT, WIDTH};
use registers::Registers;
use std::error::Error;
use std::fmt;
use tracing::{debug, info, warn};
use wram::Wram;

mod apu;
mod clock;
mod dma;
mod header;
mod memory;
mod ppu;
mod registers;
mod wram;

// TODO: Overscan
const VBLANK_LINE: u16 = 225;

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
        self.core.bus().ppu.pixels()
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        self.core.bus_mut().ready = false;

        while !self.core.bus().ready {
            self.core.step();
            debug!("{}", self.core);
        }
    }
}

pub struct Hardware {
    clock: Clock,
    mdr: u8,
    interrupt: Interrupt,
    ready: bool,
    pages: [Page; TOTAL_PAGES],
    rom: MirrorVec<u8>,
    sram: MirrorVec<u8>,
    wram: Wram,
    regs: Registers,
    dma: Dma,
    ppu: Ppu,
    apu: Apu,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>, ipl_rom: Vec<u8>) -> Self {
        let header = header::parse(&rom_data);

        info!("Title: {}", header.title);
        info!("Mapper: {:?}", header.mapper);
        info!("ROM Size: {}", header.rom_size);
        info!("SRAM Size: {}", header.sram_size);

        let pages = memory::map(&header);

        Self {
            clock: Clock::new(),
            mdr: 0,
            interrupt: 0,
            ready: false,
            pages,
            rom: MirrorVec::resize(rom_data),
            sram: MirrorVec::new(header.sram_size),
            wram: Wram::new(),
            regs: Registers::new(),
            dma: Dma::new(),
            ppu: Ppu::new(),
            apu: Apu::new(ipl_rom),
        }
    }

    fn step(&mut self, cycles: u64) {
        self.clock.add_cycles(cycles);

        while let Some(event) = self.clock.event() {
            match event {
                Event::HBlank => {
                    let line = self.clock.line();

                    if line > 0 && line < VBLANK_LINE {
                        self.ppu.draw_line(line)
                    }
                }
                Event::NewLine => {
                    let line = self.clock.line();

                    if line == VBLANK_LINE {
                        self.ready = true;
                        self.regs.set_nmi_occurred(true);

                        if self.regs.nmi_raised() {
                            self.interrupt |= INT_NMI;
                        }
                    } else if line == 0 {
                        self.ppu.start_frame();
                        self.regs.set_nmi_occurred(false);
                        self.interrupt &= !INT_NMI;
                    }
                }
            }
        }
    }

    fn read_bus_a(&mut self, address: u32) -> u8 {
        self.mdr = match self.pages[(address >> 13) as usize] {
            Page::Rom(offset) => self.rom[(offset | (address & 0x1fff)) as usize],
            Page::Sram(offset) => self.sram[(offset | (address & 0x1fff)) as usize],
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
                0x0300 => self.dma.read(address as u8, self.mdr),
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

        self.mdr
    }

    fn write_bus_a(&mut self, address: u32, value: u8) {
        self.mdr = value;

        match self.pages[(address >> 13) as usize] {
            Page::Rom(..) => warn!("Write to ROM area: {:06X} <= {:02X}", address, value),
            Page::Sram(offset) => self.sram[(offset | (address & 0x1fff)) as usize] = value,
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
                0x0300 => self.dma.write(address as u8, value),
                _ => warn!(
                    "Unmapped external register write: {:06X} <= {:02X}",
                    address, value
                ),
            },
            Page::OpenBus => warn!("Unmapped Bus A write: {:06X} <= {:02X}", address, value),
        }
    }

    fn read_bus_b(&mut self, address: u8) -> u8 {
        self.mdr = match address & 0xc0 {
            0x00 => self.ppu.read(&self.clock, address),
            0x40 => {
                self.apu.run_until(self.clock.cycles());
                self.apu.read(address)
            }
            0x80 => self.wram.read_register(address, self.mdr),
            _ => {
                warn!("Unmapped Bus B read: {:02X}", address);
                self.mdr
            }
        };

        self.mdr
    }

    fn write_bus_b(&mut self, address: u8, value: u8) {
        self.mdr = value;

        match address & 0xc0 {
            0x00 => self.ppu.write(address, value),
            0x40 => {
                self.apu.run_until(self.clock.cycles());
                self.apu.write(address, value);
            }
            0x80 => self.wram.write_register(address, value),
            _ => warn!("Unmapped Bus B write: {:02X} <= {:02X}", address, value),
        }
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        if self.dma.requested() {
            self.transfer_dma();
        }

        self.step(FAST_CYCLES);
    }

    fn read(&mut self, address: u32) -> u8 {
        if self.dma.requested() {
            self.transfer_dma();
        }

        self.step(self.clock.cycles_for_address(address) - 4);
        let value = self.read_bus_a(address);
        self.step(4);
        value
    }

    fn write(&mut self, address: u32, value: u8) {
        if self.dma.requested() {
            self.transfer_dma();
        }

        self.step(self.clock.cycles_for_address(address));
        self.write_bus_a(address, value);
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

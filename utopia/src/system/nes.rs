use super::System;
use crate::core::mos6502::{Bus, Core, Interrupt};
use crate::util::MirrorVec;
use cartridge::Cartridge;
use ppu::Ppu;
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};

const WRAM_SIZE: usize = 2048;

mod cartridge;
mod ppu;

pub fn create(rom_data: Vec<u8>) -> Result<Box<dyn System>, Box<dyn Error>> {
    let hw = Hardware::new(rom_data);
    let core = Core::new(hw);

    Ok(Box::new(Nes { core }))
}

pub struct Nes {
    core: Core<Hardware>,
}

impl System for Nes {
    fn width(&self) -> usize {
        ppu::WIDTH
    }

    fn height(&self) -> usize {
        ppu::HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        self.core.bus().ppu.pixels()
    }

    fn run_frame(&mut self) {
        let core = &mut self.core;

        core.bus_mut().ppu.start_frame();

        while !core.bus().ppu.ready() {
            core.step();
            debug!("{}", core);
        }
    }
}

struct Hardware {
    dma_address: Option<u8>,
    cycles: u64,
    mdr: u8,
    interrupt: Interrupt,
    cartridge: Cartridge,
    wram: MirrorVec<u8>,
    ppu: Ppu,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            dma_address: None,
            cycles: 0,
            mdr: 0,
            interrupt: 0,
            cartridge: Cartridge::new(rom_data),
            wram: MirrorVec::new(WRAM_SIZE),
            ppu: Ppu::new(),
        }
    }

    fn step(&mut self) {
        self.cycles += 1;
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);
    }

    fn transfer_dma(&mut self, base_address: u16) {
        debug!("DMA Transfer Begin");

        self.step();

        if (self.cycles & 1) != 0 {
            self.step();
        }

        self.dma_address = None;

        for index in 0..=255 {
            let address = base_address + index;
            let value = self.read(address);
            debug!("DMA Write: OAM <= {:02X} <= {:04X}", value, address);
            self.ppu.write_oam(value);
        }

        debug!("DMA Transfer End");
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u16) -> u8 {
        if let Some(dma_address) = self.dma_address {
            self.transfer_dma((dma_address as u16) << 8)
        }

        self.step();
        self.step();
        self.step();

        self.mdr = match address >> 13 {
            0 => self.wram[address as usize],
            1 => self
                .ppu
                .read(&mut self.cartridge, &mut self.interrupt, address),
            2 => match address {
                0x4016..=0x4017 => 0, // TODO: Joypad ports
                0x4000..=0x401f => 0, // TODO: APU ports
                _ => {
                    warn!("Read from unmapped address: {:04X}", address);
                    self.mdr
                }
            },
            3 => {
                //panic!("PRG RAM reads not yet implemented"),
                0
            }
            _ => self.cartridge.read_prg_rom(address),
        };

        self.mdr
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step();

        self.mdr = value;

        match address >> 13 {
            0 => self.wram[address as usize] = value,
            1 => self
                .ppu
                .write(&mut self.cartridge, &mut self.interrupt, address, value),
            2 => match address {
                0x4014 => self.dma_address = Some(value),
                0x4000..=0x401f => {
                    debug!("2A03 register write not yet implemented: {:02X}", address)
                }
                _ => warn!("Write to unmapped address: {:02X}", address),
            },
            3 => {
                //panic!("PRG RAM writes not yet implemented"),
                if address >= 0x6004 {
                    print!("{}", value as char);
                }
            }
            _ => panic!("Mapper register writes not yet implemented"),
        };

        self.step();
        self.step();
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
        write!(
            f,
            "V={} H={} T={}",
            self.ppu.line(),
            self.ppu.dot(),
            self.cycles
        )
    }
}

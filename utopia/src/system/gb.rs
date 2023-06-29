use super::{BiosLoader, System};
use crate::core::gbz80::{Bus, Core, State};
use crate::util::MirrorVec;
use cartridge::Cartridge;
use ppu::Ppu;
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};

mod cartridge;
mod ppu;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

const WRAM_SIZE: usize = 8192;
const HRAM_SIZE: usize = 128;

const M_CYCLE_LENGTH: u64 = 4;

pub fn create(
    rom_data: Vec<u8>,
    bios_loader: &impl BiosLoader,
    skip_boot: bool,
) -> Result<Box<dyn System>, Box<dyn Error>> {
    let (initial_state, bios_data) = if skip_boot {
        // TODO: This post-boot state should depend on hardware model
        let initial_state = State {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x0100,
            f: 0xb0, // TODO: H & C should depend on header checksum
        };

        (Some(initial_state), None)
    } else {
        let bios_data = bios_loader.load("dmg")?;

        (None, Some(bios_data))
    };

    // TODO: Should skip boot sequence for other hardware components as well
    let hw = Hardware::new(rom_data, bios_data);

    let core = Core::new(hw, initial_state);

    Ok(Box::new(GameBoy { core }))
}

pub struct GameBoy {
    core: Core<Hardware>,
}

impl System for GameBoy {
    fn width(&self) -> usize {
        WIDTH
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        &PIXELS
    }

    fn run_frame(&mut self) {
        let core = &mut self.core;

        core.bus_mut().ppu.start_frame();

        while !core.bus().ppu.ready() {
            debug!("{}", core);
            core.step();
        }
    }
}

struct Hardware {
    cycles: u64,
    interrupt_flag: u8,
    interrupt_enable: u8,
    hram: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    cartridge: Cartridge,
    ppu: Ppu,
    bios_data: Option<MirrorVec<u8>>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>, bios_data: Option<Vec<u8>>) -> Self {
        Self {
            cycles: 0,
            interrupt_flag: 0xe0,
            interrupt_enable: 0xe0,
            hram: MirrorVec::new(HRAM_SIZE),
            wram: MirrorVec::new(WRAM_SIZE),
            cartridge: Cartridge::new(rom_data),
            ppu: Ppu::new(),
            bios_data: bios_data.map(MirrorVec::from),
        }
    }

    fn step(&mut self) {
        self.cycles += M_CYCLE_LENGTH;
        self.ppu.step(M_CYCLE_LENGTH);
    }

    fn read_high_impl(&mut self, address: u8) -> u8 {
        match address {
            0x00 => 0xff, // TODO: Joypad
            0x01..=0x0e => panic!("System register read {:02X} not yet implemented", address),
            0x0f => self.interrupt_flag,
            0x10..=0x3f => panic!("APU register reads not yet implemented"),
            0x40..=0x4f => self.ppu.read(address),
            0x50..=0x7f => panic!("Unmapped register read"),
            0x80..=0xfe => self.hram[address as usize],
            0xff => self.interrupt_enable,
        }
    }

    fn write_high_impl(&mut self, address: u8, value: u8) {
        match address {
            0x01 => print!("{}", value as char),
            0x00..=0x0e => warn!("System register write {:02X} not yet implemented", address),
            0x0f => {
                self.interrupt_flag = 0xe0 | (value & 0x1f);
                debug!("Interrupt Flag: {:02X}", self.interrupt_flag);
            }
            0x10..=0x3f => warn!("APU register writes not yet implemented"),
            0x40..=0x4f => self.ppu.write(address, value),
            0x50 => {
                self.bios_data = None;
                debug!("BIOS disabled");
            }
            0x51..=0x7f => warn!("Unmapped register write: {:02X}", address),
            0x80..=0xfe => self.hram[address as usize] = value,
            0xff => {
                self.interrupt_enable = 0xe0 | (value & 0x1f);
                debug!("Interrupt Enable: {:02X}", self.interrupt_enable);
            }
        }
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        self.step();
    }

    fn read(&mut self, address: u16) -> u8 {
        self.step();

        match address >> 13 {
            0 => {
                if address < 0x0100 {
                    if let Some(bios_data) = &self.bios_data {
                        bios_data[address as usize]
                    } else {
                        self.cartridge.read_rom(address as usize)
                    }
                } else {
                    self.cartridge.read_rom(address as usize)
                }
            }
            1 | 2 | 3 => self.cartridge.read_rom(address as usize),
            4 => panic!("VRAM reads not yet implemented"),
            5 => panic!("ERAM reads not yet implemented"),
            6 => self.wram[address as usize],
            7 => match address {
                0xff00..=0xffff => self.read_high_impl(address as u8),
                0xfe00..=0xfe9f => panic!("OAM reads not yet implemented"),
                0xfea0..=0xfeff => 0xff,
                _ => panic!("Read from unmapped location"),
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step();

        match address >> 13 {
            0 | 1 | 2 | 3 => debug!("Mapper writes not yet implemented"),
            4 => debug!("VRAM writes not yet implemented"),
            5 => debug!("ERAM writes not yet implemented"),
            6 => self.wram[address as usize] = value,
            7 => match address {
                0xff00..=0xffff => self.write_high_impl(address as u8, value),
                0xfe00..=0xfe9f => debug!("OAM writes not yet implemented"),
                0xfea0..=0xfeff => (),
                _ => warn!("Write to unmapped location"),
            },
            _ => unreachable!(),
        }
    }

    fn read_high(&mut self, address: u8) -> u8 {
        self.step();
        self.read_high_impl(address)
    }

    fn write_high(&mut self, address: u8, value: u8) {
        self.step();
        self.write_high_impl(address, value);
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "T={} V={} H={}",
            self.cycles,
            self.ppu.line(),
            self.ppu.dot()
        )
    }
}

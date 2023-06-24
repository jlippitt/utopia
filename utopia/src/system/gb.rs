use super::{BiosLoader, System};
use crate::core::gbz80::{Bus, Core};
use crate::util::MirrorVec;
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const PIXELS: [u8; 0] = [];

const HRAM_SIZE: usize = 128;

pub fn create(
    rom_data: Vec<u8>,
    bios_loader: &impl BiosLoader,
) -> Result<Box<dyn System>, Box<dyn Error>> {
    let bios_data = Some(bios_loader.load("dmg")?);

    let hw = Hardware::new(rom_data, bios_data);
    let core = Core::new(hw);

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

        loop {
            core.step();
            debug!("{}", core);
        }
    }
}

struct Hardware {
    rom_data: MirrorVec<u8>,
    bios_data: Option<MirrorVec<u8>>,
    hram: MirrorVec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>, bios_data: Option<Vec<u8>>) -> Self {
        Self {
            rom_data: rom_data.into(),
            bios_data: bios_data.map(MirrorVec::from),
            hram: MirrorVec::new(HRAM_SIZE),
        }
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        //
    }

    fn read(&mut self, address: u16) -> u8 {
        match address >> 13 {
            0 => {
                if address < 0x0100 {
                    if let Some(bios_data) = &self.bios_data {
                        bios_data[address as usize]
                    } else {
                        self.rom_data[address as usize]
                    }
                } else {
                    self.rom_data[address as usize]
                }
            }
            1 | 2 | 3 => self.rom_data[address as usize],
            4 => panic!("VRAM reads not yet implemented"),
            5 => panic!("ERAM reads not yet implemented"),
            6 => panic!("WRAM reads not yet implemented"),
            7 => match address {
                0xff00..=0xffff => self.read_high(address as u8),
                0xfe00..=0xfea0 => panic!("OAM reads not yet implemented"),
                _ => panic!("Read from unmapped location"),
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address >> 13 {
            0 | 1 | 2 | 3 => panic!("Mapper writes not yet implemented"),
            4 => warn!("VRAM writes not yet implemented"),
            5 => warn!("ERAM writes not yet implemented"),
            6 => warn!("WRAM writes not yet implemented"),
            7 => match address {
                0xff00..=0xffff => self.write_high(address as u8, value),
                0xfe00..=0xfea0 => warn!("OAM writes not yet implemented"),
                _ => panic!("Write from unmapped location"),
            },
            _ => unreachable!(),
        }
    }

    fn read_high(&mut self, address: u8) -> u8 {
        match address {
            0x00..=0x0f => panic!("System register reads not yet implemented"),
            0x10..=0x3f => panic!("APU register reads not yet implemented"),
            0x40..=0x4f => panic!("PPU register reads not yet implemented"),
            0x50..=0x7f => panic!("Misc register reads not yet implemented"),
            0x80..=0xfe => self.hram[address as usize],
            0xff => panic!("Interrupt register reads not yet implemented"),
        }
    }

    fn write_high(&mut self, address: u8, value: u8) {
        match address {
            0x00..=0x0f => warn!("System register writes not yet implemented"),
            0x10..=0x3f => warn!("APU register writes not yet implemented"),
            0x40..=0x4f => warn!("PPU register writes not yet implemented"),
            0x50..=0x7f => warn!("Misc register writes not yet implemented"),
            0x80..=0xfe => self.hram[address as usize] = value,
            0xff => warn!("Interrupt register writes not yet implemented"),
        }
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

use super::{JoypadState, System};
use crate::core::wdc65c816::{Bus, Core};
use crate::util::MirrorVec;
use memory::{Page, TOTAL_PAGES};
use std::error::Error;
use std::fmt;
use tracing::{debug, warn};

mod memory;

const WIDTH: usize = 512;
const HEIGHT: usize = 448;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

const WRAM_SIZE: usize = 131072;

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
    mdr: u8,
    pages: [Page; TOTAL_PAGES],
    rom: MirrorVec<u8>,
    wram: MirrorVec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let pages = memory::map();

        Self {
            mdr: 0,
            pages,
            rom: rom_data.into(),
            wram: MirrorVec::new(WRAM_SIZE),
        }
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u32) -> u8 {
        self.mdr = match self.pages[(address >> 13) as usize] {
            Page::Rom(offset) => self.rom[(offset | (address & 0x1fff)) as usize],
            Page::Wram(offset) => self.wram[(offset | (address & 0x1fff)) as usize],
            Page::ExternalRegisters => panic!("External register reads not yet implemented"),
            Page::InternalRegisters => panic!("Internal register reads not yet implemented"),
            Page::OpenBus => {
                warn!("Read from unmapped area: {:06X}", address);
                self.mdr
            }
        };

        self.mdr
    }

    fn write(&mut self, address: u32, value: u8) {
        self.mdr = value;

        match self.pages[(address >> 13) as usize] {
            Page::Rom(..) => warn!("Write to ROM area: {:06X}", address),
            Page::Wram(offset) => self.wram[(offset | (address & 0x1fff)) as usize] = value,
            Page::ExternalRegisters => (), // TODO
            Page::InternalRegisters => (), // TODO
            Page::OpenBus => warn!("Write to unmapped area: {:06X}", address),
        }
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

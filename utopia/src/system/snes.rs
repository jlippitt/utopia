use super::{JoypadState, System};
use crate::core::wdc65c816::{Bus, Core};
use memory::{Page, TOTAL_PAGES};
use std::error::Error;
use std::fmt;

mod memory;

const WIDTH: usize = 512;
const HEIGHT: usize = 448;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

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
        println!("{}", self.core);
    }
}

pub struct Hardware {
    rom: Vec<u8>,
    pages: [Page; TOTAL_PAGES],
}

impl Hardware {
    pub fn new(rom: Vec<u8>) -> Self {
        let pages = memory::map(&rom);

        Self { rom, pages }
    }
}

impl Bus for Hardware {
    //
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

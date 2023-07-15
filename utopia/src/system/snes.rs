use super::{JoypadState, System};
use std::error::Error;

const WIDTH: usize = 512;
const HEIGHT: usize = 448;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

pub struct Snes {
    //
}

impl Snes {
    pub fn new(_rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        Ok(Snes {})
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
        //
    }
}

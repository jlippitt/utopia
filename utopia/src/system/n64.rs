use super::System;
use crate::JoypadState;
use std::error::Error;
use tracing::info;

mod header;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const PIXELS: [u8; WIDTH * 4 * HEIGHT] = [0; WIDTH * 4 * HEIGHT];

pub struct N64 {
    //core: Core<Hardware>,
}

impl N64 {
    pub fn new(rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let header = header::parse(&rom_data);

        info!("Title: {}", header.title);
        info!("Boot Address: {:08X}", header.boot_address);

        //let hw = Hardware::new(rom_data);
        //let core = Core::new(hw);
        Ok(N64 {})
    }
}

impl System for N64 {
    fn width(&self) -> usize {
        // TODO: Support for multiple resolutions
        // (Needs front-end changes!)
        WIDTH
    }

    fn height(&self) -> usize {
        // TODO: Support for multiple resolutions
        // (Needs front-end changes!)
        HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        &PIXELS
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        // TODO
    }
}

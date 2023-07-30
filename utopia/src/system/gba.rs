use super::System;
use crate::core::arm7tdmi::{Bus, Core};
use crate::util::MirrorVec;
use crate::JoypadState;
use std::error::Error;
use tracing::info;

const WIDTH: usize = 240;
const HEIGHT: usize = 160;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

pub struct GameBoyAdvance {
    core: Core<Hardware>,
}

impl GameBoyAdvance {
    pub fn new(rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let hw = Hardware::new(rom_data);
        let core = Core::new(hw);
        Ok(GameBoyAdvance { core })
    }
}

impl System for GameBoyAdvance {
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
        let core = &mut self.core;

        loop {
            core.step();
        }
    }
}

struct Hardware {
    _rom: MirrorVec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        let title = String::from_utf8_lossy(&rom_data[0xa0..=0xab]).into_owned();
        info!("Title: {}", title);

        Self {
            _rom: rom_data.into(),
        }
    }
}

impl Bus for Hardware {}

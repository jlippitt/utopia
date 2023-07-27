use super::System;
use crate::core::mips::{Bus, Core, State};
use crate::util::Primitive;
use crate::JoypadState;
use std::error::Error;
use tracing::info;

mod header;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const PIXELS: [u8; WIDTH * 4 * HEIGHT] = [0; WIDTH * 4 * HEIGHT];

const IPL3_START_ADDRESS: u32 = 0xA4000040;

pub struct N64 {
    core: Core<Hardware>,
}

impl N64 {
    pub fn new(rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let header = header::parse(&rom_data);

        info!("Title: {}", header.title);
        info!("Boot Address: {:08X}", header.boot_address);

        let hw = Hardware::new(rom_data);

        let core = Core::new(
            hw,
            State {
                pc: IPL3_START_ADDRESS,
                ..Default::default()
            },
        );

        Ok(N64 { core })
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
        // TODO: Timing
        loop {
            self.core.step();
        }
    }
}

struct Hardware {
    rom: Vec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self { rom: rom_data }
    }

    fn read_physical<T: Primitive>(&mut self, address: u32) -> T {
        // TODO
        Default::default()
    }
}

impl Bus for Hardware {
    fn read<T: Primitive>(&mut self, address: u32) -> T {
        // TODO: Cache
        match address >> 29 {
            4 | 5 => self.read_physical::<T>(address),
            _ => todo!("TLB"),
        }
    }
}

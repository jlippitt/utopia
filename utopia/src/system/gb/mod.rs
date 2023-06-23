use crate::core::gbz80::{Bus, Core};
use super::{System, BiosLoader};
use std::error::Error;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const PIXELS: [u8; 0] = [];

pub fn create(rom_data: Vec<u8>, bios_loader: &impl BiosLoader) -> Result<Box<dyn System>, Box<dyn Error>> {
    let bios_data = bios_loader.load("dmg")?;

    let hw = Hardware::new(rom_data, bios_data);
    let core = Core::new(hw);

    Ok(Box::new(GameBoy { _core: core }))
}

pub struct GameBoy {
    _core: Core<Hardware>,
}

impl System for GameBoy {
    fn width(&self) -> usize { WIDTH }

    fn height(&self) -> usize { HEIGHT }

    fn pixels(&self) -> &[u8] { &PIXELS }

    fn run_frame(&mut self) {
        panic!("Game Boy not yet implemented");

        // let core = &mut self.core;
        //
        // loop {
        //     core.step();
        //     debug!("{}", core);
        // }
    }
}

struct Hardware {
    _rom_data: Vec<u8>,
    _bios_data: Vec<u8>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>, bios_data: Vec<u8>) -> Self {
        Self {
            _rom_data: rom_data,
            _bios_data: bios_data,
        }
    }
}

impl Bus for Hardware {
    //
}
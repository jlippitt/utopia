use super::System;
use crate::core::mos6502::Core;
use hardware::Hardware;
use std::error::Error;

mod hardware;
mod rom;

pub fn create(rom_data: Vec<u8>) -> Result<Box<dyn System>, Box<dyn Error>> {
    let hw = Hardware::new(rom_data);
    let core = Core::new(hw);

    Ok(Box::new(NES { core }))
}

pub struct NES {
    core: Core<Hardware>,
}

impl System for NES {
    fn run(&mut self) {
        self.core.step();
    }
}

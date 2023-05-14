use super::System;
use bus::Bus;
use std::error::Error;

mod bus;
mod rom;

pub struct NES {
    bus: Bus,
}

pub fn create(rom_data: Vec<u8>) -> Result<Box<dyn System>, Box<dyn Error>> {
    let bus = Bus::new(rom_data);

    Ok(Box::new(NES { bus }))
}

impl System for NES {
    fn run(&mut self) {
        println!("Here");
    }
}

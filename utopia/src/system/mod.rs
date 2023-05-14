use std::error::Error;

mod nes;

pub trait System {
    fn run(&mut self);
}

pub fn create(extension: &str, rom_data: Vec<u8>) -> Result<Box<dyn System>, Box<dyn Error>> {
    let constructor = match extension {
        "nes" => Ok(nes::create),
        _ => Err("ROM type not supported".to_owned()),
    };

    constructor?(rom_data)
}

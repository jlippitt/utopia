use std::error::Error;
use std::path::Path;

mod gb;
mod nes;

pub trait System {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn run_frame(&mut self);
}

pub trait BiosLoader {
    fn load(&self, name: &str) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub fn create(rom_path: &str, rom_data: Vec<u8>, bios_loader: &impl BiosLoader) -> Result<Box<dyn System>, Box<dyn Error>> {
    let extension = Path::new(rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "gb" => gb::create(rom_data, bios_loader),
        "nes" => nes::create(rom_data),
        _ => Err("ROM type not supported".to_owned())?,
    }
}

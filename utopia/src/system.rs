use std::error::Error;
use std::path::Path;

mod gb;
mod nes;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct JoypadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
}

pub trait System {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn run_frame(&mut self, joypad_state: &JoypadState);

    fn clip_top(&self) -> usize {
        0
    }

    fn clip_bottom(&self) -> usize {
        0
    }
}

pub trait BiosLoader {
    fn load(&self, name: &str) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct Options {
    pub skip_boot: bool,
}

pub fn create(
    rom_path: &str,
    rom_data: Vec<u8>,
    bios_loader: &impl BiosLoader,
    options: &Options,
) -> Result<Box<dyn System>, Box<dyn Error>> {
    let extension = Path::new(rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "gb" => gb::create(rom_data, bios_loader, options.skip_boot),
        "nes" => nes::create(rom_data),
        _ => Err("ROM type not supported".to_owned())?,
    }
}

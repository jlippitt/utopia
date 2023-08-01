use gb::GameBoy;
use gba::GameBoyAdvance;
use n64::N64;
use nes::Nes;
use snes::Snes;
use std::error::Error;
use std::path::Path;

mod gb;
mod gba;
mod n64;
mod nes;
mod snes;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct JoypadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub l: bool,
    pub r: bool,
    pub select: bool,
    pub start: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Sync {
    Video,
    Audio,
}

pub trait System {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn run_frame(&mut self, joypad_state: &JoypadState);

    fn sync(&self) -> Sync {
        Sync::Video
    }

    fn sample_rate(&self) -> u64 {
        44100
    }

    fn total_samples(&self) -> u64 {
        0
    }

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

    Ok(match extension {
        "gb" => Box::new(GameBoy::new(rom_data, bios_loader, options.skip_boot)?),
        "gba" => Box::new(GameBoyAdvance::new(rom_data, bios_loader)?),
        "nes" => Box::new(Nes::new(rom_data)?),
        "sfc" | "smc" => Box::new(Snes::new(rom_data, bios_loader)?),
        "z64" => Box::new(N64::new(rom_data)?),
        _ => Err("ROM type not supported".to_owned())?,
    })
}

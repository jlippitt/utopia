use crate::util::mirror::MirrorableMut;
use gb::GameBoy;
use gba::GameBoyAdvance;
use n64::N64;
use nes::Nes;
use snes::Snes;
use std::collections::VecDeque;
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

pub type AudioQueue = VecDeque<(f32, f32)>;

pub trait System {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixels(&self) -> &[u8];
    fn run_frame(&mut self, joypad_state: &JoypadState);

    fn sample_rate(&self) -> u64 {
        44100
    }

    fn audio_queue(&mut self) -> Option<&mut AudioQueue> {
        None
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

pub trait Mapped: MirrorableMut<Output = u8> {}

impl<T: MirrorableMut<Output = u8>> Mapped for T {}

pub trait MemoryMapper {
    type Mapped: Mapped;
    fn open(&self, len: usize, battery_backed: bool) -> Result<Self::Mapped, Box<dyn Error>>;
}

pub struct Options<T: MemoryMapper, U: BiosLoader> {
    pub memory_mapper: T,
    pub bios_loader: U,
    pub skip_boot: bool,
}

pub fn create<T: MemoryMapper + 'static, U: BiosLoader>(
    rom_data: Vec<u8>,
    rom_path: &str,
    options: &Options<T, U>,
) -> Result<Box<dyn System>, Box<dyn Error>> {
    let extension = Path::new(rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    Ok(match extension {
        "gb" => Box::new(GameBoy::<T::Mapped>::new(rom_data, options)?),
        "gba" => Box::new(GameBoyAdvance::new(
            rom_data,
            &options.bios_loader,
            options.skip_boot,
        )?),
        "nes" => Box::new(Nes::new(rom_data, &options.memory_mapper)?),
        "sfc" | "smc" => Box::new(Snes::new(rom_data, options)?),
        "z64" => Box::new(N64::new(rom_data)?),
        _ => Err("ROM type not supported".to_owned())?,
    })
}

use crate::{BiosLoader, CreateOptions, MemoryMapper};
use gb::GameBoy;
use gba::GameBoyAdvance;
use n64::N64;
use nes::Nes;
use snes::Snes;
use std::collections::VecDeque;
use std::error;

pub mod gb;
pub mod gba;
pub mod n64;
pub mod nes;
pub mod snes;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct JoypadState {
    pub buttons: [bool; 17],
    pub axes: [i32; 4],
}

pub type AudioQueue = VecDeque<(f32, f32)>;

pub trait System {
    fn pixels(&self) -> &[u8];
    fn pitch(&self) -> usize;
    fn run_frame(&mut self, joypad_state: &JoypadState);

    fn sample_rate(&self) -> u64 {
        44100
    }

    fn audio_queue(&mut self) -> Option<&mut AudioQueue> {
        None
    }

    fn screen_width(&self) -> u32 {
        (self.pitch() / 4).try_into().unwrap()
    }

    fn screen_height(&self) -> u32 {
        (self.pixels().len() / self.pitch()).try_into().unwrap()
    }

    fn screen_resolution(&self) -> (u32, u32) {
        (self.screen_width(), self.screen_height())
    }
}

pub fn create<T: MemoryMapper + 'static, U: BiosLoader>(
    extension: &str,
    rom_data: Vec<u8>,
    options: &CreateOptions<T, U>,
) -> Result<Box<dyn System>, Box<dyn error::Error>> {
    Ok(match extension {
        "gb" => Box::new(GameBoy::<T::Mapped>::new(rom_data, options)?),
        "gba" => Box::new(GameBoyAdvance::new(
            rom_data,
            &options.bios_loader,
            options.skip_boot,
        )?),
        "n64" | "z64" => Box::new(N64::new(rom_data)?),
        "nes" => Box::new(Nes::new(rom_data, &options.memory_mapper)?),
        "sfc" | "smc" => Box::new(Snes::new(rom_data, options)?),
        _ => Err("ROM type not supported".to_owned())?,
    })
}

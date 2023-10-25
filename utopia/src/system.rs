use crate::{BiosLoader, Error, MemoryMapper};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;

pub mod gb;
pub mod gba;
pub mod nes;
pub mod snes;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct JoypadState {
    pub buttons: [bool; 17],
    pub axes: [i32; 4],
}

pub type AudioQueue = VecDeque<(f32, f32)>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SystemType {
    GameBoy,
    GameBoyAdvance,
    Nes,
    Snes,
}

impl TryFrom<&Path> for SystemType {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Error> {
        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
            .unwrap_or("".to_owned());

        match extension.as_str() {
            "gb" | "gbc" => Ok(Self::GameBoy),
            "gba" => Ok(Self::GameBoyAdvance),
            "nes" => Ok(Self::Nes),
            "sfc" | "smc" => Ok(Self::Snes),
            _ => Err(format!("No system found for file extension '.{}'", extension).into()),
        }
    }
}

pub struct SystemOptions<'a, T: MemoryMapper> {
    pub system_type: SystemType,
    pub bios_loader: &'a dyn BiosLoader,
    pub memory_mapper: &'a T,
    pub skip_boot: bool,
}

pub trait System<T: MemoryMapper> {
    fn create_instance(&self, options: InstanceOptions) -> Result<Box<dyn Instance>, Error>;
    fn default_resolution(&self) -> (u32, u32);

    fn default_sample_rate(&self) -> Option<u64> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct WgpuContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub texture: Arc<wgpu::Texture>,
}

#[derive(Debug)]
pub struct InstanceOptions {
    pub rom_data: Vec<u8>,
    pub wgpu_context: Option<WgpuContext>,
}

pub trait Instance {
    fn run_frame(&mut self, joypad_state: &JoypadState);

    fn resolution(&self) -> (u32, u32);
    fn pixels(&self) -> &[u8];

    fn sample_rate(&self) -> u64 {
        44100
    }

    fn audio_queue(&mut self) -> Option<&mut AudioQueue> {
        None
    }

    fn pitch(&self) -> usize {
        self.resolution().0 as usize * 4
    }
}

pub fn create<'a, T: MemoryMapper + 'static>(
    options: SystemOptions<'a, T>,
) -> Result<Box<dyn System<T> + 'a>, Error> {
    Ok(match options.system_type {
        SystemType::GameBoy => Box::new(gb::System::new(options)),
        SystemType::GameBoyAdvance => Box::new(gba::System::new(options)),
        SystemType::Nes => Box::new(nes::System::new(options)),
        SystemType::Snes => Box::new(snes::System::new(options)),
    })
}

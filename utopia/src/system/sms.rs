use super::{InstanceOptions, JoypadState, MemoryMapper, Size, SystemOptions, WgpuContext};
use crate::core::z80::{self, Core};
use crate::util::mirror::Mirror;
use std::fmt;
use std::marker::PhantomData;
use tracing::trace;

pub struct System<T: MemoryMapper + 'static> {
    _phantom: PhantomData<T>,
}

impl<T: MemoryMapper> System<T> {
    pub fn new(_options: SystemOptions<T>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: MemoryMapper> crate::System<T> for System<T> {
    fn default_output_resolution(&self) -> Size {
        Size::new(256, 192)
    }

    fn default_sample_rate(&self) -> Option<u64> {
        None
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        Ok(Box::new(Instance::new(options)))
    }
}

pub struct Instance {
    core: Core<Bus>,
    _wgpu_context: WgpuContext,
}

impl Instance {
    pub fn new(options: InstanceOptions) -> Self {
        let bus = Bus::new(options.rom_data);
        let core = Core::new(bus);

        Self {
            core,
            _wgpu_context: options.wgpu_context,
        }
    }
}

impl crate::Instance for Instance {
    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        let core = &mut self.core;

        loop {
            trace!("{}", core);
            core.step();
        }
    }

    fn present(&self, _canvas: &wgpu::Texture) {
        // TODO: Render pixels to WGPU texture
    }
}

pub struct Bus {
    cycles: u64,
    rom: Mirror<Box<[u8]>>,
    ram: Mirror<Box<[u8]>>,
}

impl Bus {
    const RAM_SIZE: usize = 8192;

    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            cycles: 0,
            rom: rom_data.into_boxed_slice().into(),
            ram: vec![0; Self::RAM_SIZE].into_boxed_slice().into(),
        }
    }

    fn read_memory(&mut self, address: u16) -> u8 {
        match address >> 14 {
            0..=2 => self.rom[address as usize],
            _ => self.ram[address as usize],
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        match address >> 14 {
            0..=2 => panic!("Write to ROM area"),
            _ => {
                if address >= 0xfffc {
                    todo!("Mapping registers");
                }

                self.ram[address as usize] = value;
            }
        }
    }
}

impl z80::Bus for Bus {
    fn idle(&mut self) {
        self.cycles += 1;
    }

    fn fetch(&mut self, address: u16) -> u8 {
        self.cycles += 2;
        let value = self.read_memory(address);
        self.cycles += 2;
        value
    }

    fn read(&mut self, address: u16) -> u8 {
        self.cycles += 2;
        let value = self.read_memory(address);
        self.cycles += 1;
        value
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cycles += 3;
        self.write_memory(address, value);
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={}", self.cycles)
    }
}

use super::{InstanceOptions, JoypadState, MemoryMapper, Size, SystemOptions, WgpuContext};
use crate::core::m68000::{self, Core};
use crate::util::memory::{Masked, Memory, Reader, Value, Writer};
use std::marker::PhantomData;
use tracing::warn;
use vdp::Vdp;

mod vdp;

const RAM_SIZE: usize = 65536;

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
            core.step();
        }
    }

    fn present(&self, _canvas: &wgpu::Texture) {
        // TODO: Render pixels to WGPU texture
    }
}

struct Bus {
    rom: Memory,
    ram: Memory,
    vdp: Vdp,
}

impl Bus {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            rom: rom_data.into(),
            ram: Memory::new(RAM_SIZE),
            vdp: Vdp::new(),
        }
    }
}

impl m68000::Bus for Bus {
    fn read<T: Value>(&self, address: u32) -> T {
        match (address >> 16) as u8 {
            0x00..=0x3f => self.rom.read_be(address as usize),
            0xa1 => self.read_be(address),
            0xc0 => self.vdp.read_be(address),
            0xff => self.ram.read_be(address as usize & 0xffff),
            _ => panic!("Unmapped read: {:06X}", address),
        }
    }

    fn write<T: Value>(&mut self, address: u32, value: T) {
        match (address >> 16) as u8 {
            0xa1 => self.write_be(address, value),
            0xc0 => self.vdp.write_be(address, value),
            0xff => self.ram.write_be(address as usize & 0xffff, value),
            _ => panic!("Unmapped write: {:06X} <= {:04X}", address, value),
        }
    }
}

impl Reader for Bus {
    type Value = u16;

    fn read_register(&self, address: u32) -> u16 {
        match address as u16 {
            // TODO: Hardware version?
            0x0000 => 0x80,
            // TODO: Port 1 control
            0x0008 => 0,
            // TODO: Port 2 control
            0x000a => 0,
            // TODO: EXT port control
            0x000c => 0,
            // TODO: Z80 status
            0x1100 => 0,
            port => unimplemented!("Port read {:04X}", port),
        }
    }
}

impl Writer for Bus {
    fn write_register(&mut self, address: u32, value: Masked<u16>) {
        match address as u16 {
            port => warn!("Unmapped port write {:04X} <= {:04X}", port, value.get()),
        }
    }
}

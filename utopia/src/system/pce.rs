use crate::core::huc6280::{Bus, Core, Interrupt};
use crate::{Error, InstanceOptions, JoypadState, MemoryMapper, WgpuContext};
use std::fmt;

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 224;

pub struct System;

impl<T: MemoryMapper> crate::System<T> for System {
    fn create_instance(&self, options: InstanceOptions) -> Result<Box<dyn crate::Instance>, Error> {
        Ok(Box::new(Instance::new(options)))
    }

    fn default_resolution(&self) -> (u32, u32) {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }
}

pub struct Instance {
    wgpu_context: Option<WgpuContext>,
    _core: Core<Hardware>,
}

impl Instance {
    fn new(options: InstanceOptions) -> Self {
        Self {
            wgpu_context: options.wgpu_context,
            _core: Core::new(Hardware::new(options.rom_data)),
        }
    }
}

impl crate::Instance for Instance {
    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        //
    }

    fn wgpu_context(&self) -> &WgpuContext {
        self.wgpu_context.as_ref().unwrap()
    }

    fn wgpu_context_mut(&mut self) -> &mut WgpuContext {
        self.wgpu_context.as_mut().unwrap()
    }

    fn resolution(&self) -> (u32, u32) {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    // Effectively deprecated. TODO: Remove from interface.
    fn pixels(&self) -> &[u8] {
        &[]
    }
}

struct Hardware {
    _rom_data: Vec<u8>,
}

impl Hardware {
    fn new(rom_data: Vec<u8>) -> Self {
        Self {
            _rom_data: rom_data,
        }
    }
}

impl Bus for Hardware {
    fn read(&mut self, _address: u16) -> u8 {
        0
    }

    fn write(&mut self, _address: u16, _value: u8) {
        //
    }

    fn poll(&mut self) -> Interrupt {
        0
    }

    fn acknowledge(&mut self, _interrupt: Interrupt) {
        //
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

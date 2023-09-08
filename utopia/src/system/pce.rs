use crate::core::huc6280::{Bus, Core, Interrupt};
use crate::util::MirrorVec;
use crate::{Error, InstanceOptions, JoypadState, MemoryMapper, SystemOptions, WgpuContext};
use std::fmt;
use tracing::{debug, warn};

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 224;

const WRAM_SIZE: usize = 8192;

pub struct System;

impl System {
    pub fn new(_options: SystemOptions<'_, impl MemoryMapper>) -> Self {
        Self
    }
}

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
    core: Core<Hardware>,
}

impl Instance {
    fn new(options: InstanceOptions) -> Self {
        Self {
            wgpu_context: options.wgpu_context,
            core: Core::new(Hardware::new(options.rom_data)),
        }
    }
}

impl crate::Instance for Instance {
    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        loop {
            self.core.step();
            debug!("{}", self.core);
        }
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
    rom_data: Vec<u8>,
    wram: MirrorVec<u8>,
}

impl Hardware {
    fn new(rom_data: Vec<u8>) -> Self {
        Self {
            rom_data,
            wram: MirrorVec::new(WRAM_SIZE),
        }
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u32) -> u8 {
        match (address >> 13) & 0xff {
            0x00..=0x7f => self.rom_data[address as usize],
            0xf7 => todo!("SRAM Reads"),
            0xf8 => self.wram[address as usize & 0x1fff],
            0xff => {
                match address & 0x1fff {
                    0x1000 => 0, // TODO: Joypad
                    port => todo!("I/O Port Read: {:04X}", port),
                }
            }
            _ => panic!("Read from unmapped address {:04X}", address),
        }
    }

    fn write(&mut self, address: u32, value: u8) {
        match (address >> 13) & 0xff {
            0xf7 => todo!("SRAM Writes"),
            0xf8 => self.wram[address as usize & 0x1fff] = value,
            0xff => warn!("I/O Port Writes"),
            _ => panic!("Read from unmapped address {:04X}", address),
        }
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

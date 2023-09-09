use crate::core::huc6280::{Bus, Core, Interrupt};
use crate::util::MirrorVec;
use crate::{Error, InstanceOptions, JoypadState, MemoryMapper, SystemOptions, WgpuContext};
use std::fmt;
use tracing::{debug, warn};
use vdc::Vdc;
use vde::Vde;

mod vdc;
mod vde;

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 224;

const WRAM_SIZE: usize = 8192;

const SLOW_CYCLES: u64 = 12;
const FAST_CYCLES: u64 = 3;

pub struct System;

impl System {
    pub fn new(_options: SystemOptions<'_, impl MemoryMapper>) -> Self {
        Self
    }
}

impl<T: MemoryMapper> crate::System<T> for System {
    fn default_resolution(&self) -> (u32, u32) {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    fn create_instance(&self, options: InstanceOptions) -> Result<Box<dyn crate::Instance>, Error> {
        Ok(Box::new(Instance::new(options)))
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
    fn resolution(&self) -> (u32, u32) {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    // Effectively deprecated. TODO: Remove from interface.
    fn pixels(&self) -> &[u8] {
        &[]
    }

    fn wgpu_context(&self) -> &WgpuContext {
        self.wgpu_context.as_ref().unwrap()
    }

    fn wgpu_context_mut(&mut self) -> &mut WgpuContext {
        self.wgpu_context.as_mut().unwrap()
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        loop {
            self.core.step();
            debug!("{}", self.core);
        }
    }
}

struct Hardware {
    clock_speed: u64,
    mdr: u8,
    rom_data: Vec<u8>,
    wram: MirrorVec<u8>,
    vdc: Vdc,
    vde: Vde,
}

impl Hardware {
    fn new(rom_data: Vec<u8>) -> Self {
        Self {
            clock_speed: SLOW_CYCLES,
            mdr: 0,
            rom_data,
            wram: MirrorVec::new(WRAM_SIZE),
            vdc: Vdc::new(),
            vde: Vde::new(),
        }
    }

    fn step(&mut self) {
        self.vde.step(self.clock_speed);
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u32) -> u8 {
        self.step();

        self.mdr = match (address >> 13) & 0xff {
            0x00..=0x7f => self.rom_data[address as usize],
            0xf7 => todo!("SRAM Reads"),
            0xf8 => self.wram[address as usize & 0x1fff],
            0xff => match address & 0x1c00 {
                0x0000 => self.vdc.read(address as u16 & 0x03ff, self.mdr),
                0x0400 => self.vde.read(address as u16 & 0x03ff, self.mdr),
                0x1000 => 0, // TODO: Joypad
                _ => panic!("Unmapped I/O port read: {:04X}", address),
            },
            _ => panic!("Read from unmapped address {:06X}", address),
        };

        self.mdr
    }

    fn write(&mut self, address: u32, value: u8) {
        self.step();

        self.mdr = value;

        match (address >> 13) & 0xff {
            0xf7 => todo!("SRAM Writes"),
            0xf8 => self.wram[address as usize & 0x1fff] = value,
            0xff => match address & 0x1c00 {
                0x0000 => self.vdc.write(address as u16 & 0x03ff, value),
                0x0400 => self.vde.write(address as u16 & 0x03ff, value),
                _ => warn!("Unmapped I/O port write: {:04X} <= {:02X}", address, value),
            },
            _ => panic!("Read from unmapped address {:06X}", address),
        }
    }

    fn poll(&mut self) -> Interrupt {
        0
    }

    fn acknowledge(&mut self, _interrupt: Interrupt) {
        //
    }

    fn set_clock_speed(&mut self, clock_speed_high: bool) {
        self.clock_speed = if clock_speed_high {
            FAST_CYCLES
        } else {
            SLOW_CYCLES
        };

        debug!("Clock Speed: {}", self.clock_speed);
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={} V={}", self.vde.cycles(), self.vde.line_counter())
    }
}

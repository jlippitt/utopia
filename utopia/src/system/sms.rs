use super::{InstanceOptions, JoypadState, MemoryMapper, Size, SystemOptions, WgpuContext};
use crate::core::z80::{self, Core};
use crate::util::mirror::Mirror;
use std::fmt;
use std::marker::PhantomData;
use tracing::{trace, warn};
use vdp::Vdp;

mod vdp;

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
    mdr: u8,
    page_offset: [usize; 3],
    rom: Mirror<Box<[u8]>>,
    ram: Mirror<Box<[u8]>>,
    vdp: Vdp,
}

impl Bus {
    const RAM_SIZE: usize = 8192;

    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            cycles: 0,
            mdr: 0,
            page_offset: [0, 16384, 32768],
            rom: rom_data.into_boxed_slice().into(),
            ram: vec![0; Self::RAM_SIZE].into_boxed_slice().into(),
            vdp: Vdp::new(),
        }
    }

    fn read_memory(&mut self, address: u16) -> u8 {
        match address >> 14 {
            0 => {
                if address < 0x0400 {
                    self.rom[address as usize]
                } else {
                    self.rom[self.page_offset[0] + (address as usize & 0x3fff)]
                }
            }
            1 => self.rom[self.page_offset[1] + (address as usize & 0x3fff)],
            2 => self.rom[self.page_offset[2] + (address as usize & 0x3fff)],
            _ => self.ram[address as usize],
        }
    }
}

impl z80::Bus for Bus {
    fn idle(&mut self, cycles: u64) {
        self.cycles += cycles;
    }

    fn fetch(&mut self, address: u16) -> u8 {
        self.cycles += 2;
        self.mdr = self.read_memory(address);
        self.cycles += 2;
        self.mdr
    }

    fn read(&mut self, address: u16) -> u8 {
        self.cycles += 2;
        self.mdr = self.read_memory(address);
        self.cycles += 1;
        self.mdr
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cycles += 3;
        self.mdr = value;

        match address >> 14 {
            0..=2 => panic!("Write to ROM area"),
            _ => {
                self.ram[address as usize] = value;

                match address {
                    0xfffc => todo!("Save RAM"),
                    0xfffd..=0xffff => {
                        let index = address as usize - 0xfffd;
                        self.page_offset[index] = ((value as usize) << 14) & (self.rom.len() - 1);
                        trace!(
                            "Mapper Page {}: {} ({})",
                            index,
                            value,
                            self.page_offset[index]
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    fn read_port(&mut self, address: u16) -> u8 {
        self.cycles += 3;

        self.mdr = match address & 0xc1 {
            // 0x40 => self.vdp.v_counter(),
            // 0x41 => self.vdp.h_counter(),
            // 0x80 => self.vdp.read_data(),
            // 0x81 => self.vdp.status(),
            0xc0 | 0xc1 => 0, // TODO: Joypad/Country Code
            _ => {
                warn!("Unmapped Port Read: {:02X}", address as u8);
                self.mdr
            }
        };

        self.cycles += 1;
        self.mdr
    }

    fn write_port(&mut self, address: u16, value: u8) {
        self.cycles += 4;
        self.mdr = value;

        match address & 0xc1 {
            0x00 => warn!("TODO: Memory control"),
            0x01 => warn!("TODO: I/O Control"),
            0x40 | 0x41 => (), // TODO: PSG
            0x80 => self.vdp.write_data(value),
            0x81 => self.vdp.write_command(value),
            _ => (),
        }
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={}", self.cycles)
    }
}

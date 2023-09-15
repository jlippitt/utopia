use crate::core::huc6280::{self, Bus, Core};
use crate::util::mirror::Mirror;
use crate::util::{gfx, MirrorVec};
use crate::{InstanceOptions, JoypadState, Mapped, MemoryMapper, SystemOptions, WgpuContext};
use interrupt::Interrupt;
use std::error::Error;
use std::fmt;
use tracing::{debug, info, warn};
use vce::Vce;
use vdc::Vdc;

mod interrupt;
mod vce;
mod vdc;

const SRAM_SIZE: usize = 8192;
const WRAM_SIZE: usize = 8192;

const SLOW_CYCLES: u64 = 12;
const FAST_CYCLES: u64 = 3;

pub struct System<'a, T: MemoryMapper + 'static> {
    memory_mapper: &'a T,
}

impl<'a, T: MemoryMapper> System<'a, T> {
    pub fn new(options: SystemOptions<'a, T>) -> Self {
        Self {
            memory_mapper: options.memory_mapper,
        }
    }
}

impl<'a, T: MemoryMapper> crate::System<T> for System<'a, T> {
    fn default_resolution(&self) -> (u32, u32) {
        (Vdc::DEFAULT_WIDTH as u32, Vdc::DEFAULT_HEIGHT as u32)
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        let result = Instance::new(self.memory_mapper, options);

        Ok(Box::new(
            result.map_err(|err| crate::Error(err.to_string()))?,
        ))
    }
}

pub struct Instance<T: Mapped> {
    wgpu_context: Option<WgpuContext>,
    core: Core<Hardware<T>>,
}

impl<T: Mapped> Instance<T> {
    fn new(
        memory_mapper: &impl MemoryMapper<Mapped = T>,
        options: InstanceOptions,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            wgpu_context: options.wgpu_context,
            core: Core::new(Hardware::new(options.rom_data, memory_mapper)?),
        })
    }
}

impl<T: Mapped> crate::Instance for Instance<T> {
    fn resolution(&self) -> (u32, u32) {
        let vdc = &self.core.bus().vdc;
        (vdc.display_width() as u32, vdc.display_height() as u32)
    }

    fn pixels(&self) -> &[u8] {
        self.core.bus().vdc.pixels()
    }

    fn wgpu_context(&self) -> &WgpuContext {
        self.wgpu_context.as_ref().unwrap()
    }

    fn wgpu_context_mut(&mut self) -> &mut WgpuContext {
        self.wgpu_context.as_mut().unwrap()
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        self.core.bus_mut().vde.start_frame();

        while !self.core.bus().vde.frame_done() {
            self.core.step();
            debug!("{}", self.core);
        }

        if let Some(wgpu_context) = &self.wgpu_context {
            gfx::write_pixels_to_texture(
                wgpu_context,
                &wgpu_context.texture,
                self.pixels(),
                self.pitch(),
            )
        }
    }
}

struct Hardware<T: Mapped> {
    cycles: u64,
    clock_speed: u64,
    mdr: u8,
    interrupt: Interrupt,
    rom: MirrorVec<u8>,
    sram: Mirror<T>,
    wram: MirrorVec<u8>,
    vdc: Vdc,
    vde: Vce,
}

impl<T: Mapped> Hardware<T> {
    fn new(
        mut rom_data: Vec<u8>,
        memory_mapper: &impl MemoryMapper<Mapped = T>,
    ) -> Result<Self, Box<dyn Error>> {
        let rom_size = rom_data.len();

        info!("ROM Size: {}", rom_size);

        if rom_size == 0x060000 {
            // 384K ROM gets split into 256K + 128K parts
            let mut split_rom = Vec::with_capacity(0x100000);
            split_rom.extend_from_slice(&rom_data[0x000000..0x040000]);
            split_rom.extend_from_slice(&rom_data[0x000000..0x040000]);
            split_rom.extend_from_slice(&rom_data[0x040000..0x060000]);
            split_rom.extend_from_slice(&rom_data[0x040000..0x060000]);
            split_rom.extend_from_slice(&rom_data[0x040000..0x060000]);
            split_rom.extend_from_slice(&rom_data[0x040000..0x060000]);
            rom_data = split_rom;
        }

        let interrupt = Interrupt::new();

        Ok(Self {
            cycles: 0,
            clock_speed: SLOW_CYCLES,
            mdr: 0,
            rom: rom_data.into(),
            sram: memory_mapper.open(SRAM_SIZE, true)?.into(),
            wram: MirrorVec::new(WRAM_SIZE),
            vdc: Vdc::new(interrupt.clone()),
            vde: Vce::new(),
            interrupt,
        })
    }

    fn step(&mut self) {
        self.cycles += self.clock_speed;
        self.vde.step(&mut self.vdc, self.clock_speed);
    }
}

impl<T: Mapped> Bus for Hardware<T> {
    fn read(&mut self, address: u32) -> u8 {
        self.step();

        self.mdr = match (address >> 13) & 0xff {
            0x00..=0x7f => self.rom[address as usize],
            0xf7 => self.sram[address as usize & 0x1fff],
            0xf8 => self.wram[address as usize & 0x1fff],
            0xff => match address & 0x1c00 {
                0x0000 => self.vdc.read(address as u16 & 0x03ff, self.mdr),
                0x0400 => self.vde.read(address as u16 & 0x03ff, self.mdr),
                0x0800 => 0, // TODO: PSG
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
            0xf7 => self.sram[address as usize & 0x1fff] = value,
            0xf8 => self.wram[address as usize & 0x1fff] = value,
            0xff => match address & 0x1c00 {
                0x0000 => self.vdc.write(address as u16 & 0x03ff, value),
                0x0400 => self.vde.write(address as u16 & 0x03ff, value),
                _ => warn!("Unmapped I/O port write: {:04X} <= {:02X}", address, value),
            },
            _ => panic!("Read from unmapped address {:06X}", address),
        }
    }

    fn poll(&mut self) -> huc6280::Interrupt {
        self.interrupt.poll()
    }

    fn acknowledge(&mut self, interrupt: huc6280::Interrupt) {
        self.interrupt.clear(interrupt.into());
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

impl<T: Mapped> fmt::Display for Hardware<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={} V={}", self.cycles, self.vde.line_counter())
    }
}

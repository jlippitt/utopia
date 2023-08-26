use crate::core::sm83::{Bus, Core, State};
use crate::util::gfx;
use crate::util::mirror::MirrorVec;
use crate::{
    AudioQueue, BiosLoader, InstanceOptions, JoypadState, Mapped, MemoryMapper, SystemOptions,
    WgpuContext,
};
use apu::Apu;
use cartridge::Cartridge;
use interrupt::Interrupt;
use joypad::Joypad;
use ppu::Ppu;
use std::error::Error;
use std::fmt;
use timer::Timer;
use tracing::{debug, warn};

mod apu;
mod cartridge;
mod interrupt;
mod joypad;
mod ppu;
mod timer;

const WRAM_SIZE: usize = 8192;
const HRAM_SIZE: usize = 128;

const M_CYCLE_LENGTH: u64 = 4;

pub struct System<T: BiosLoader, U: MemoryMapper + 'static> {
    bios_loader: T,
    memory_mapper: U,
    skip_boot: bool,
}

impl<T: BiosLoader, U: MemoryMapper> System<T, U> {
    pub fn new(options: SystemOptions<T, U>) -> Self {
        Self {
            bios_loader: options.bios_loader,
            memory_mapper: options.memory_mapper,
            skip_boot: options.skip_boot,
        }
    }
}

impl<T: BiosLoader, U: MemoryMapper> crate::System<T, U> for System<T, U> {
    fn default_resolution(&self) -> (u32, u32) {
        (ppu::WIDTH as u32, ppu::HEIGHT as u32)
    }

    fn default_sample_rate(&self) -> Option<u64> {
        Some(Apu::SAMPLE_RATE)
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        let result = Instance::new(
            &self.bios_loader,
            &self.memory_mapper,
            self.skip_boot,
            options,
        );

        Ok(Box::new(
            result.map_err(|err| crate::Error(err.to_string()))?,
        ))
    }
}

pub struct Instance<T: Mapped> {
    core: Core<Hardware<T>>,
    wgpu_context: Option<WgpuContext>,
}

impl<T: Mapped> Instance<T> {
    pub fn new<U: BiosLoader, V: MemoryMapper<Mapped = T>>(
        bios_loader: &U,
        memory_mapper: &V,
        skip_boot: bool,
        options: InstanceOptions,
    ) -> Result<Self, Box<dyn Error>> {
        let bios_data = if !skip_boot {
            bios_loader.load("dmg_boot").ok()
        } else {
            None
        };

        let initial_state = bios_data.is_none().then_some(
            // TODO: This post-boot state should depend on hardware model
            State {
                a: 0x01,
                b: 0x00,
                c: 0x13,
                d: 0x00,
                e: 0xd8,
                h: 0x01,
                l: 0x4d,
                sp: 0xfffe,
                pc: 0x0100,
                f: 0xb0, // TODO: H & C should depend on header checksum
            },
        );

        // TODO: Should skip boot sequence for other hardware components as well
        let hw = Hardware::new(memory_mapper, skip_boot, options.rom_data, bios_data)?;

        let core = Core::new(hw, initial_state);

        Ok(Instance {
            core,
            wgpu_context: options.wgpu_context,
        })
    }
}

impl<T: Mapped> crate::Instance for Instance<T> {
    fn resolution(&self) -> (u32, u32) {
        (ppu::WIDTH as u32, ppu::HEIGHT as u32)
    }

    fn pixels(&self) -> &[u8] {
        self.core.bus().ppu.pixels()
    }

    fn sample_rate(&self) -> u64 {
        Apu::SAMPLE_RATE
    }

    fn audio_queue(&mut self) -> Option<&mut AudioQueue> {
        Some(self.core.bus_mut().apu.audio_queue())
    }

    fn wgpu_context(&self) -> &WgpuContext {
        self.wgpu_context.as_ref().unwrap()
    }

    fn wgpu_context_mut(&mut self) -> &mut WgpuContext {
        self.wgpu_context.as_mut().unwrap()
    }

    fn run_frame(&mut self, joypad_state: &JoypadState) {
        let core = &mut self.core;

        core.bus_mut().joypad.update(joypad_state);
        core.bus_mut().ppu.start_frame();

        while !core.bus().ppu.ready() {
            debug!("{}", core);
            core.step();
        }

        if let Some(wgpu_context) = &self.wgpu_context {
            gfx::write_pixels_to_texture(wgpu_context, self.pixels(), self.pitch())
        }
    }
}

struct Hardware<T: Mapped> {
    cycles: u64,
    dma_address: Option<u16>,
    interrupt: Interrupt,
    timer: Timer,
    hram: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    cartridge: Cartridge<T>,
    ppu: Ppu,
    apu: Apu,
    joypad: Joypad,
    bios_data: Option<MirrorVec<u8>>,
}

impl<T: Mapped> Hardware<T> {
    pub fn new<U: MemoryMapper<Mapped = T>>(
        memory_mapper: &U,
        skip_boot: bool,
        rom_data: Vec<u8>,
        bios_data: Option<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            cycles: 0,
            dma_address: None,
            interrupt: Interrupt::new(),
            timer: Timer::new(),
            hram: MirrorVec::new(HRAM_SIZE),
            wram: MirrorVec::new(WRAM_SIZE),
            cartridge: Cartridge::new(rom_data, memory_mapper)?,
            ppu: Ppu::new(skip_boot),
            apu: Apu::new(),
            joypad: Joypad::new(),
            bios_data: bios_data.map(MirrorVec::from),
        })
    }

    fn step(&mut self) -> bool {
        self.cycles += M_CYCLE_LENGTH;
        self.timer
            .step(&mut self.interrupt, &mut self.apu, M_CYCLE_LENGTH);
        self.ppu.step(&mut self.interrupt, M_CYCLE_LENGTH);

        if (self.cycles & 3) == 0 {
            self.apu.step();
        }

        let Some(src_address) = self.dma_address else {
            return false;
        };

        let dst_address = src_address as u8;
        let value = self.read_normal(src_address);

        debug!(
            "DMA Transfer: FE{:02X} <= {:02X} <= {:04X}",
            dst_address, value, src_address
        );

        self.ppu.write_oam(dst_address, value);

        self.dma_address = if (dst_address + 1) <= 0x9f {
            Some(src_address + 1)
        } else {
            None
        };

        true
    }

    fn read_normal(&mut self, address: u16) -> u8 {
        match address >> 13 {
            0 => {
                if address < 0x0100 {
                    if let Some(bios_data) = &self.bios_data {
                        bios_data[address as usize]
                    } else {
                        self.cartridge.read_rom(address)
                    }
                } else {
                    self.cartridge.read_rom(address)
                }
            }
            1 | 2 | 3 => self.cartridge.read_rom(address),
            4 => self.ppu.read_vram(address),
            5 => self.cartridge.read_ram(address),
            6 => self.wram[address as usize],
            7 => match address {
                0xff00..=0xffff => self.read_high_normal(address as u8),
                0xfe00..=0xfe9f => self.ppu.read_oam(address as u8),
                0xfea0..=0xfeff => 0xff,
                _ => {
                    warn!("Read WRAM mirror: {:04X}", address);
                    self.wram[address as usize]
                }
            },
            _ => unreachable!(),
        }
    }

    fn read_restricted(&mut self, address: u16) -> u8 {
        match address >> 13 {
            7 => match address {
                0xff00..=0xffff => self.read_high_restricted(address as u8),
                _ => 0xff,
            },
            _ => 0xff,
        }
    }

    fn write_normal(&mut self, address: u16, value: u8) {
        match address >> 13 {
            0 | 1 | 2 | 3 => self.cartridge.write_register(address, value),
            4 => self.ppu.write_vram(address, value),
            5 => self.cartridge.write_ram(address, value),
            6 => self.wram[address as usize] = value,
            7 => match address {
                0xff00..=0xffff => self.write_high_normal(address as u8, value),
                0xfe00..=0xfe9f => self.ppu.write_oam(address as u8, value),
                0xfea0..=0xfeff => (),
                _ => warn!("Write to WRAM mirror: {:04X} <= {:02X}", address, value),
            },
            _ => unreachable!(),
        }
    }

    fn write_restricted(&mut self, address: u16, value: u8) {
        match address >> 13 {
            7 => match address {
                0xff00..=0xffff => self.write_high_restricted(address as u8, value),
                _ => (),
            },
            _ => (),
        }
    }

    fn read_high_normal(&mut self, address: u8) -> u8 {
        match address {
            0x00 => self.joypad.read(),
            0x04..=0x07 => self.timer.read(address),
            0x0f => self.interrupt.flag(),
            0x10..=0x3f => self.apu.read(address),
            0x40..=0x4f => self.ppu.read_register(address),
            0x80..=0xfe => self.hram[address as usize],
            0xff => self.interrupt.enable(),
            _ => {
                warn!("Unmapped register read: {:02X}", address);
                0xff
            }
        }
    }

    fn read_high_restricted(&mut self, address: u8) -> u8 {
        match address {
            0x80..=0xfe => self.hram[address as usize],
            _ => 0xff,
        }
    }

    fn write_high_normal(&mut self, address: u8, value: u8) {
        match address {
            0x00 => self.joypad.write(value),
            0x01 | 0x02 => (), // TODO: Serial port
            0x04..=0x07 => self.timer.write(&mut self.apu, address, value),
            0x0f => self.interrupt.set_flag(value),
            0x10..=0x3f => self.apu.write(address, value),
            0x46 => self.dma_address = Some((value as u16) << 8),
            0x40..=0x4f => self.ppu.write_register(&mut self.interrupt, address, value),
            0x50 => {
                self.bios_data = None;
                debug!("BIOS disabled");
            }
            0x80..=0xfe => self.hram[address as usize] = value,
            0xff => self.interrupt.set_enable(value),
            _ => warn!("Unmapped register write: {:02X} <= {:02X}", address, value),
        }
    }

    fn write_high_restricted(&mut self, address: u8, value: u8) {
        match address {
            0x80..=0xfe => self.hram[address as usize] = value,
            _ => (),
        }
    }
}

impl<T: Mapped> Bus for Hardware<T> {
    fn idle(&mut self) {
        self.step();
    }

    fn read(&mut self, address: u16) -> u8 {
        if self.step() {
            self.read_restricted(address)
        } else {
            self.read_normal(address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if self.step() {
            self.write_restricted(address, value);
        } else {
            self.write_normal(address, value);
        }
    }

    fn read_high(&mut self, address: u8) -> u8 {
        if self.step() {
            self.read_high_restricted(address)
        } else {
            self.read_high_normal(address)
        }
    }

    fn write_high(&mut self, address: u8, value: u8) {
        if self.step() {
            self.write_high_restricted(address, value);
        } else {
            self.write_high_normal(address, value);
        }
    }

    fn poll(&self) -> u8 {
        self.interrupt.poll()
    }

    fn acknowledge(&mut self, mask: u8) {
        self.interrupt.acknowledge(mask);
    }
}

impl<T: Mapped> fmt::Display for Hardware<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "V={} H={} T={}",
            self.ppu.line(),
            self.ppu.dot(),
            self.cycles,
        )
    }
}

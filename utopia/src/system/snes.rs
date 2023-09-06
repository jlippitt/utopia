use crate::core::wdc65c816::{Bus, Core, Interrupt, INT_NMI};
use crate::util::gfx;
use crate::util::mirror::{Mirror, MirrorVec};
use crate::{
    BiosLoader, InstanceOptions, JoypadState, Mapped, MemoryMapper, SystemOptions, WgpuContext,
};
use apu::Apu;
use clock::{Clock, Event, FAST_CYCLES, TIMER_IRQ};
use dma::Dma;
use joypad::Joypad;
use memory::{Page, TOTAL_PAGES};
use ppu::Ppu;
use registers::Registers;
use std::error::Error;
use std::fmt;
use tracing::{debug, info, warn};
use wram::Wram;

mod apu;
mod clock;
mod dma;
mod header;
mod joypad;
mod memory;
mod ppu;
mod registers;
mod wram;

const SAMPLE_RATE: u64 = 32000;

pub enum CoprocessorType {
    None,
    Dsp,
}

pub struct System<'a, U: MemoryMapper + 'static> {
    bios_loader: &'a dyn BiosLoader,
    memory_mapper: &'a U,
}

impl<'a, T: MemoryMapper> System<'a, T> {
    pub fn new(options: SystemOptions<'a, T>) -> Self {
        Self {
            bios_loader: options.bios_loader,
            memory_mapper: options.memory_mapper,
        }
    }
}

impl<'a, T: MemoryMapper> crate::System<T> for System<'a, T> {
    fn default_resolution(&self) -> (u32, u32) {
        (ppu::WIDTH as u32, ppu::HEIGHT as u32)
    }

    fn default_sample_rate(&self) -> Option<u64> {
        Some(SAMPLE_RATE)
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        let result = Instance::new(self.bios_loader, self.memory_mapper, options);

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
    pub fn new<U: MemoryMapper<Mapped = T>>(
        bios_loader: &dyn BiosLoader,
        memory_mapper: &U,
        options: InstanceOptions,
    ) -> Result<Self, Box<dyn Error>> {
        let hw = Hardware::new(bios_loader, memory_mapper, options.rom_data)?;
        let core = Core::new(hw);

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
        SAMPLE_RATE
    }

    fn wgpu_context(&self) -> &WgpuContext {
        self.wgpu_context.as_ref().unwrap()
    }

    fn wgpu_context_mut(&mut self) -> &mut WgpuContext {
        self.wgpu_context.as_mut().unwrap()
    }

    fn audio_queue(&mut self) -> Option<&mut crate::AudioQueue> {
        Some(self.core.bus_mut().apu.audio_queue())
    }

    fn run_frame(&mut self, joypad_state: &JoypadState) {
        let core = &mut self.core;
        core.bus_mut().joypad.update(joypad_state);
        core.bus_mut().ready = false;

        while !core.bus().ready {
            core.step();
            debug!("{}", core);
        }

        let cpu_cycles = core.bus().clock.cycles();
        core.bus_mut().apu.run_until(cpu_cycles);

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

pub struct Hardware<T: Mapped> {
    clock: Clock,
    mdr: u8,
    interrupt: Interrupt,
    ready: bool,
    pages: [Page; TOTAL_PAGES],
    rom: MirrorVec<u8>,
    sram: Mirror<T>,
    wram: Wram,
    regs: Registers,
    dma: Dma,
    ppu: Ppu,
    apu: Apu,
    joypad: Joypad,
}

impl<T: Mapped> Hardware<T> {
    pub fn new<U: MemoryMapper<Mapped = T>>(
        bios_loader: &dyn BiosLoader,
        memory_mapper: &U,
        rom_data: Vec<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        let ipl_rom = bios_loader.load("ipl_rom")?;

        let header = header::parse(&rom_data);
        info!("Title: {}", header.title);
        info!("Map Mode: {:02X}", header.map_mode);
        info!("Cartridge Type: {:02X}", header.cartridge_type);
        info!("ROM Size: {}", header.rom_size);
        info!("SRAM Size: {}", header.sram_size);

        let battery_backed = [0x02, 0x05].contains(&(header.cartridge_type & 0x0f));
        info!("Battery Backed: {}", battery_backed);

        let coprocessor_type = if (header.cartridge_type & 0x0f) > 0x03 {
            match header.cartridge_type & 0xf0 {
                0x00 => CoprocessorType::Dsp,
                value => unimplemented!("Coprocessor Type: {:02X}", value),
            }
        } else {
            CoprocessorType::None
        };

        let pages = memory::map(&header, coprocessor_type);

        Ok(Self {
            clock: Clock::new((header.map_mode & 0x10) != 0),
            mdr: 0,
            interrupt: 0,
            ready: false,
            pages,
            rom: MirrorVec::resize(rom_data),
            sram: memory_mapper.open(header.sram_size, battery_backed)?.into(),
            wram: Wram::new(),
            regs: Registers::new(),
            dma: Dma::new(),
            ppu: Ppu::new(),
            apu: Apu::new(ipl_rom),
            joypad: Joypad::new(),
        })
    }

    fn step(&mut self, cycles: u64) {
        self.clock.add_cycles(cycles);

        while let Some(event) = self.clock.event() {
            match event {
                Event::HBlank => {
                    let line = self.clock.line();

                    if line < self.ppu.vblank_line() {
                        self.transfer_hdma();

                        if line >= 1 {
                            self.ppu.draw_line(
                                line,
                                self.clock.interlace(),
                                self.clock.odd_frame(),
                            );
                        }
                    }
                }
                Event::NewLine => {
                    let line = self.clock.line();

                    if line == self.ppu.vblank_line() {
                        self.ready = true;
                        self.clock.set_nmi_occurred(&mut self.interrupt, true);
                        self.ppu.on_vblank_start();
                    } else if line == 0 {
                        self.clock.set_nmi_occurred(&mut self.interrupt, false);
                        self.interrupt &= !INT_NMI;
                        self.ppu.on_frame_start();
                        self.init_hdma();
                    } else if line == (self.ppu.vblank_line() + 3) {
                        // TODO: Actual timine for this
                        self.joypad.perform_auto_read();
                    }
                }
                Event::Irq => self.interrupt |= TIMER_IRQ,
            }
        }
    }

    fn read_bus_a(&mut self, address: u32) -> u8 {
        self.mdr = match self.pages[(address >> 13) as usize] {
            Page::Rom(offset) => self.rom[(offset | (address & 0x1fff)) as usize],
            Page::Sram(offset) => self.sram[(offset | (address & 0x1fff)) as usize],
            Page::Wram(offset) => self.wram[(offset | (address & 0x1fff)) as usize],
            Page::ExternalRegisters => match address & 0x1f00 {
                0x0100 => self.read_bus_b(address as u8),
                _ => {
                    warn!("Unmapped external register read: {:06X}", address);
                    self.mdr
                }
            },
            Page::InternalRegisters => match address & 0x1f00 {
                0x0000 => self.joypad.read_serial(address as u8, self.mdr),
                0x0200 => self.read_register(address as u8, self.mdr),
                0x0300 => self.dma.read(address as u8, self.mdr),
                _ => {
                    warn!("Unmapped internal register read: {:06X}", address);
                    self.mdr
                }
            },
            Page::Coprocessor(..) => todo!("Coprocessor Read"),
            Page::OpenBus => {
                warn!("Unmapped Bus A read: {:06X}", address);
                self.mdr
            }
        };

        self.mdr
    }

    fn write_bus_a(&mut self, address: u32, value: u8) {
        self.mdr = value;

        match self.pages[(address >> 13) as usize] {
            Page::Rom(..) => warn!("Write to ROM area: {:06X} <= {:02X}", address, value),
            Page::Sram(offset) => self.sram[(offset | (address & 0x1fff)) as usize] = value,
            Page::Wram(offset) => self.wram[(offset | (address & 0x1fff)) as usize] = value,
            Page::ExternalRegisters => match address & 0x1f00 {
                0x0100 => self.write_bus_b(address as u8, value),
                _ => warn!(
                    "Unmapped external register write: {:06X} <= {:02X}",
                    address, value
                ),
            },
            Page::InternalRegisters => match address & 0x1f00 {
                0x0000 => self.joypad.write_serial(address as u8, value),
                0x0200 => self.write_register(address as u8, value),
                0x0300 => self.dma.write(address as u8, value),
                _ => warn!(
                    "Unmapped external register write: {:06X} <= {:02X}",
                    address, value
                ),
            },
            Page::Coprocessor(..) => todo!("Coprocessor Write"),
            Page::OpenBus => warn!("Unmapped Bus A write: {:06X} <= {:02X}", address, value),
        }
    }

    fn read_bus_b(&mut self, address: u8) -> u8 {
        self.mdr = match address & 0xc0 {
            0x00 => self.ppu.read(&self.clock, address),
            0x40 => {
                self.apu.run_until(self.clock.cycles());
                self.apu.read(address)
            }
            0x80 => self.wram.read_register(address, self.mdr),
            _ => {
                warn!("Unmapped Bus B read: {:02X}", address);
                self.mdr
            }
        };

        self.mdr
    }

    fn write_bus_b(&mut self, address: u8, value: u8) {
        self.mdr = value;

        match address & 0xc0 {
            0x00 => self.ppu.write(&mut self.clock, address, value),
            0x40 => {
                self.apu.run_until(self.clock.cycles());
                self.apu.write(address, value);
            }
            0x80 => self.wram.write_register(address, value),
            _ => warn!("Unmapped Bus B write: {:02X} <= {:02X}", address, value),
        }
    }
}

impl<T: Mapped> Bus for Hardware<T> {
    fn idle(&mut self) {
        if self.dma.requested() {
            self.transfer_dma();
        }

        self.step(FAST_CYCLES);
    }

    fn read(&mut self, address: u32) -> u8 {
        if self.dma.requested() {
            self.transfer_dma();
        }

        self.step(self.clock.cycles_for_address(address) - 4);
        let value = self.read_bus_a(address);
        self.step(4);
        value
    }

    fn write(&mut self, address: u32, value: u8) {
        if self.dma.requested() {
            self.transfer_dma();
        }

        self.step(self.clock.cycles_for_address(address));
        self.write_bus_a(address, value);
    }

    fn poll(&self) -> Interrupt {
        self.interrupt
    }

    fn acknowledge(&mut self, interrupt: Interrupt) {
        self.interrupt &= !interrupt;
    }
}

impl<T: Mapped> fmt::Display for Hardware<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.clock)
    }
}

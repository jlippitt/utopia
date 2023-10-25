use crate::core::mos6502::{self, Bus, Core};
use crate::util::upscaler::Upscaler;
use crate::util::MirrorVec;
use crate::{AudioQueue, Error, InstanceOptions, JoypadState, Mapped, MemoryMapper, SystemOptions};
use apu::Apu;
use bitflags::bitflags;
use cartridge::Cartridge;
use interrupt::Interrupt;
use joypad::Joypad;
use ppu::Ppu;
use std::fmt;
use tracing::debug;

const WRAM_SIZE: usize = 2048;
const CLIP_LINES: usize = 8;
const WIDTH: u32 = ppu::WIDTH as u32;
const HEIGHT: u32 = (ppu::HEIGHT - CLIP_LINES * 2) as u32;

mod apu;
mod cartridge;
mod interrupt;
mod joypad;
mod ppu;

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
        (WIDTH, HEIGHT)
    }

    fn default_sample_rate(&self) -> Option<u64> {
        Some(Apu::SAMPLE_RATE)
    }

    fn create_instance(&self, options: InstanceOptions) -> Result<Box<dyn crate::Instance>, Error> {
        Ok(Box::new(Instance::new(self.memory_mapper, options)?))
    }
}

pub struct Instance<T: Mapped> {
    core: Core<Hardware<T>>,
    upscaler: Upscaler,
}

impl<T: Mapped> Instance<T> {
    pub fn new(
        memory_mapper: &impl MemoryMapper<Mapped = T>,
        options: InstanceOptions,
    ) -> Result<Self, Error> {
        let hw = Hardware::new(options.rom_data, memory_mapper)?;
        let core = Core::new(hw);

        let mut upscaler = Upscaler::new(options.wgpu_context);
        upscaler.set_texture_size(ppu::WIDTH as u32, ppu::HEIGHT as u32);

        Ok(Instance { core, upscaler })
    }
}

impl<T: Mapped> crate::Instance for Instance<T> {
    fn resolution(&self) -> (u32, u32) {
        (WIDTH, HEIGHT)
    }

    fn pixels(&self) -> &[u8] {
        let pixels = self.core.bus().ppu.pixels();
        let start = CLIP_LINES * ppu::WIDTH * 4;
        let end = pixels.len() - start;
        &pixels[start..end]
    }

    fn sample_rate(&self) -> u64 {
        Apu::SAMPLE_RATE
    }

    fn audio_queue(&mut self) -> Option<&mut AudioQueue> {
        Some(self.core.bus_mut().apu.audio_queue())
    }

    fn run_frame(&mut self, joypad_state: &JoypadState) {
        let core = &mut self.core;

        core.bus_mut().joypad.update(joypad_state);
        core.bus_mut().ppu.start_frame();

        while !core.bus().ppu.ready() {
            core.step();
            debug!("{}", core);
        }
    }

    fn present(&self, canvas: wgpu::TextureView) {
        self.upscaler.render(canvas, self.core.bus().ppu.pixels());
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct DmaRequest: u8 {
        const OAM = 0x01;
        const DMC = 0x02;
    }
}

struct Hardware<T: Mapped> {
    dma_request: DmaRequest,
    dma_oam_src: u8,
    cycles: u64,
    mdr: u8,
    interrupt: Interrupt,
    cartridge: Cartridge<T>,
    wram: MirrorVec<u8>,
    joypad: Joypad,
    ppu: Ppu,
    apu: Apu,
}

impl<T: Mapped> Hardware<T> {
    pub fn new(
        rom_data: Vec<u8>,
        memory_mapper: &impl MemoryMapper<Mapped = T>,
    ) -> Result<Self, Error> {
        let interrupt = Interrupt::new();

        Ok(Self {
            dma_request: DmaRequest::empty(),
            dma_oam_src: 0,
            cycles: 0,
            mdr: 0,
            cartridge: Cartridge::new(rom_data, memory_mapper, interrupt.clone())?,
            wram: MirrorVec::new(WRAM_SIZE),
            joypad: Joypad::new(),
            ppu: Ppu::new(interrupt.clone()),
            apu: Apu::new(interrupt.clone()),
            interrupt,
        })
    }

    fn step_all(&mut self) {
        // PPU does 3 cycles for every 1 machine cycle
        self.step_ppu();
        self.step_ppu();
        self.step_ppu();
        self.step_others();
    }

    fn step_ppu(&mut self) {
        self.cycles += 4;
        self.ppu.step(&mut self.cartridge);
    }

    fn step_others(&mut self) {
        self.apu.step(&mut self.dma_request, &self.cartridge);
        self.cartridge.on_cpu_cycle();
    }

    fn transfer_dma(&mut self) {
        debug!("DMA Transfer Begin");

        self.step_all();

        if (self.cycles % 12) != 0 {
            self.step_all();
        }

        if self.dma_request.contains(DmaRequest::OAM) {
            self.dma_request.remove(DmaRequest::OAM);

            let base_address = (self.dma_oam_src as u16) << 8;

            for index in 0..=255 {
                if self.dma_request.contains(DmaRequest::DMC) {
                    self.load_dmc_sample();
                }

                let address = base_address + index;
                let value = self.read(address);
                debug!("DMA Write: OAM <= {:02X} <= {:04X}", value, address);
                self.ppu.write_oam(value);
            }
        } else {
            self.load_dmc_sample();
        }

        debug!("DMA Transfer End");
    }

    fn load_dmc_sample(&mut self) {
        self.dma_request.remove(DmaRequest::DMC);
        let address = self.apu.dmc_sample_address();
        let value = self.read(address);
        debug!("DMA Write: DMC <= {:02X} <= {:04X}", value, address);
        self.apu.write_dmc_sample(value);
    }
}

impl<T: Mapped> Bus for Hardware<T> {
    fn read(&mut self, address: u16) -> u8 {
        if !self.dma_request.is_empty() {
            self.transfer_dma();
        }

        self.step_ppu();
        self.step_ppu();

        self.mdr = self.cartridge.read_prg(address, self.mdr);

        self.mdr = match address >> 13 {
            0 => self.wram[address as usize],
            1 => self.ppu.read(&mut self.cartridge, address),
            2 => match address {
                0x4016..=0x4017 => self.joypad.read_register(address, self.mdr),
                0x4000..=0x401f => self.apu.read_register(address, self.mdr),
                _ => self.mdr,
            },
            _ => self.mdr,
        };

        self.step_ppu();
        self.step_others();
        self.mdr
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step_all();

        self.mdr = value;

        self.cartridge.write_prg(address, value);

        match address >> 13 {
            0 => self.wram[address as usize] = value,
            1 => self.ppu.write(&mut self.cartridge, address, value),
            2 => match address {
                0x4014 => {
                    self.dma_request.insert(DmaRequest::OAM);
                    self.dma_oam_src = value;
                }
                0x4016 => self.joypad.write_register(value),
                0x4000..=0x401f => self.apu.write_register(address, value),
                _ => (),
            },
            _ => (),
        };
    }

    fn poll(&mut self) -> mos6502::Interrupt {
        self.interrupt.poll()
    }

    fn acknowledge(&mut self, interrupt: mos6502::Interrupt) {
        self.interrupt.clear(interrupt.into());
    }
}

impl<T: Mapped> fmt::Display for Hardware<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "V={} H={} T={}",
            self.ppu.line(),
            self.ppu.dot(),
            self.cycles
        )
    }
}

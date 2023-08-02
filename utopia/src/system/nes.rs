use super::System;
use crate::core::mos6502::{Bus, Core, Interrupt};
use crate::util::MirrorVec;
use crate::AudioQueue;
use crate::JoypadState;
use apu::Apu;
use bitflags::bitflags;
use cartridge::Cartridge;
use joypad::Joypad;
use ppu::Ppu;
use std::error::Error;
use std::fmt;
use tracing::debug;

const WRAM_SIZE: usize = 2048;
const CLIP_AMOUNT: usize = 8;

mod apu;
mod cartridge;
mod joypad;
mod ppu;

pub struct Nes {
    core: Core<Hardware>,
}

impl Nes {
    pub fn new(rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let hw = Hardware::new(rom_data);
        let core = Core::new(hw);
        Ok(Nes { core })
    }
}

impl System for Nes {
    fn width(&self) -> usize {
        ppu::WIDTH
    }

    fn height(&self) -> usize {
        ppu::HEIGHT
    }

    fn clip_top(&self) -> usize {
        CLIP_AMOUNT
    }

    fn clip_bottom(&self) -> usize {
        CLIP_AMOUNT
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

    fn run_frame(&mut self, joypad_state: &JoypadState) {
        let core = &mut self.core;

        core.bus_mut().joypad.update(joypad_state);
        core.bus_mut().ppu.start_frame();

        while !core.bus().ppu.ready() {
            core.step();
            debug!("{}", core);
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct DmaRequest: u8 {
        const OAM = 0x01;
        const DMC = 0x02;
    }
}

struct Hardware {
    dma_request: DmaRequest,
    dma_oam_src: u8,
    cycles: u64,
    mdr: u8,
    interrupt: Interrupt,
    cartridge: Cartridge,
    wram: MirrorVec<u8>,
    joypad: Joypad,
    ppu: Ppu,
    apu: Apu,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            dma_request: DmaRequest::empty(),
            dma_oam_src: 0,
            cycles: 0,
            mdr: 0,
            interrupt: 0,
            cartridge: Cartridge::new(rom_data),
            wram: MirrorVec::new(WRAM_SIZE),
            joypad: Joypad::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
        }
    }

    fn step_all(&mut self) {
        // PPU does 3 cycles for every 1 machine cycle
        self.step_ppu();
        self.step_ppu();
        self.step_ppu();
        self.step_apu();
    }

    fn step_ppu(&mut self) {
        self.cycles += 4;
        self.ppu.step(&mut self.cartridge, &mut self.interrupt);
    }

    fn step_apu(&mut self) {
        self.apu.step(&mut self.interrupt, &mut self.dma_request);
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
        self.apu.write_dmc_sample(&mut self.interrupt, value);
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u16) -> u8 {
        if !self.dma_request.is_empty() {
            self.transfer_dma();
        }

        self.step_all();

        self.mdr = self.cartridge.read_prg(address, self.mdr);

        self.mdr = match address >> 13 {
            0 => self.wram[address as usize],
            1 => self
                .ppu
                .read(&mut self.cartridge, &mut self.interrupt, address),
            2 => match address {
                0x4016..=0x4017 => self.joypad.read_register(address, self.mdr),
                0x4000..=0x401f => self
                    .apu
                    .read_register(&mut self.interrupt, address, self.mdr),
                _ => self.mdr,
            },
            _ => self.mdr,
        };

        self.mdr
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step_ppu();

        self.mdr = value;

        self.cartridge.write_prg(address, value);

        match address >> 13 {
            0 => self.wram[address as usize] = value,
            1 => self
                .ppu
                .write(&mut self.cartridge, &mut self.interrupt, address, value),
            2 => match address {
                0x4014 => {
                    self.dma_request.insert(DmaRequest::OAM);
                    self.dma_oam_src = value;
                }
                0x4016 => self.joypad.write_register(value),
                0x4000..=0x401f => self.apu.write_register(&mut self.interrupt, address, value),
                _ => (),
            },
            _ => (),
        };

        self.step_ppu();
        self.step_ppu();
        self.step_apu();
    }

    fn poll(&mut self) -> Interrupt {
        self.interrupt
    }

    fn acknowledge(&mut self, interrupt: Interrupt) {
        self.interrupt &= !interrupt;
    }
}

impl fmt::Display for Hardware {
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

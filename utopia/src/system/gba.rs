use super::{BiosLoader, System};
use crate::core::arm7tdmi::{Bus, Core, Mode, State};
use crate::util::facade::{ReadFacade, Value, WriteFacade};
use crate::util::MirrorVec;
use crate::JoypadState;
use audio::Audio;
use cartridge::Cartridge;
use dma::Dma;
use ppu::Ppu;
use registers::Registers;
use std::error::Error;
use tracing::warn;

mod audio;
mod cartridge;
mod dma;
mod ppu;
mod registers;

const WIDTH: usize = 240;
const HEIGHT: usize = 160;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

const IWRAM_SIZE: usize = 32768;
const EWRAM_SIZE: usize = 262144;

pub struct GameBoyAdvance {
    core: Core<Hardware>,
}

impl GameBoyAdvance {
    pub fn new(
        rom: Vec<u8>,
        bios_loader: &impl BiosLoader,
        skip_boot: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let bios = bios_loader.load("gba_bios")?;
        let hw = Hardware::new(rom, bios);

        let mut initial_state: State = Default::default();

        if skip_boot {
            initial_state.pc = 0x0800_0000;
            initial_state.regs[13] = 0x0300_7f00;
            initial_state.bank.irq[0] = 0x0300_7fa0;
            initial_state.bank.svc[0] = 0x0300_7fe0;
            initial_state.cpsr.m = Mode::System;
        };

        let core = Core::new(hw, initial_state);
        Ok(GameBoyAdvance { core })
    }
}

impl System for GameBoyAdvance {
    fn pixels(&self) -> &[u8] {
        &PIXELS
    }

    fn pitch(&self) -> usize {
        WIDTH * 4
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        let core = &mut self.core;

        loop {
            core.step();
        }
    }
}

struct Hardware {
    cartridge: Cartridge,
    iwram: MirrorVec<u8>,
    ewram: MirrorVec<u8>,
    bios: Vec<u8>,
    regs: Registers,
    ppu: Ppu,
    audio: Audio,
    dma: Dma,
}

impl Hardware {
    pub fn new(rom: Vec<u8>, bios: Vec<u8>) -> Self {
        Self {
            cartridge: Cartridge::new(rom),
            iwram: MirrorVec::new(IWRAM_SIZE),
            ewram: MirrorVec::new(EWRAM_SIZE),
            bios,
            regs: Registers::new(),
            ppu: Ppu::new(),
            audio: Audio::new(),
            dma: Dma::new(),
        }
    }
}

impl Bus for Hardware {
    fn read<T: Value>(&mut self, address: u32) -> T {
        match address >> 24 {
            0x00 => self.bios.read_le(address as usize),
            0x02 => self.ewram.read_le(address as usize),
            0x03 => self.iwram.read_le(address as usize),
            0x04 => match address & 0x00ff_ffff {
                0x0000..=0x005f => todo!("LCD Register Reads"),
                0x0060..=0x00af => self.audio.read_le(address),
                0x00b0..=0x00ff => self.dma.read_le(address),
                0x0100..=0x011f => todo!("Timer Register Reads"),
                0x0120..=0x01ff => todo!("Serial Register Reads"),
                address => self.regs.read_le(address),
            },
            0x05 => todo!("Palette RAM Reads"),
            0x06 => self.ppu.vram().read_le(address as usize & 0x00ff_ffff),
            0x07 => todo!("OAM Reads"),
            0x08..=0x0d => self.cartridge.read_le(address),
            0xe0 => todo!("SRAM Reads"),
            _ => panic!("Unmapped Read: {:08X}", address),
        }
    }

    fn write<T: Value>(&mut self, address: u32, value: T) {
        match address >> 24 {
            0x00 => panic!("Write to BIOS area: {:08X} <= {:08X}", address, value),
            0x02 => self.ewram.write_le(address as usize, value),
            0x03 => self.iwram.write_le(address as usize, value),
            0x04 => match address & 0x00ff_ffff {
                0x0000..=0x005f => warn!("LCD Register Writes not yet implemented"),
                0x0060..=0x00af => self.audio.write_le(address, value),
                0x00b0..=0x00ff => self.dma.write_le(address, value),
                0x0100..=0x011f => warn!("Timer Register Writes not yet implemented"),
                0x0120..=0x01ff => warn!("Serial Register Writes not yet implemented"),
                address => self.regs.write_le(address, value),
            },
            0x05 => warn!("Palette RAM Writes not yet implemented"),
            0x06 => self
                .ppu
                .vram_mut()
                .write_le(address as usize & 0x00ff_ffff, value),
            0x07 => warn!("OAM Writes not yet implemented"),
            0x08..=0x0d => panic!("Write to ROM area: {:08X} <= {:08X}", address, value),
            0xe0 => warn!("SRAM Writes not yet implemented"),
            _ => panic!("Unmapped Write: {:08X} <= {:08X}", address, value),
        }
    }
}

use super::{BiosLoader, System};
use crate::core::arm7tdmi::{Bus, Core};
use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use crate::util::MirrorVec;
use crate::JoypadState;
use audio::Audio;
use dma::Dma;
use std::error::Error;
use tracing::{debug, info, warn};

mod audio;
mod dma;

const WIDTH: usize = 240;
const HEIGHT: usize = 160;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

const IWRAM_SIZE: usize = 32768;
const EWRAM_SIZE: usize = 262144;

pub struct GameBoyAdvance {
    core: Core<Hardware>,
}

impl GameBoyAdvance {
    pub fn new(rom: Vec<u8>, bios_loader: &impl BiosLoader) -> Result<Self, Box<dyn Error>> {
        let bios = bios_loader.load("gba_bios")?;
        let hw = Hardware::new(rom, bios);
        let core = Core::new(hw);
        Ok(GameBoyAdvance { core })
    }
}

impl System for GameBoyAdvance {
    fn width(&self) -> usize {
        WIDTH
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        &PIXELS
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        let core = &mut self.core;

        loop {
            core.step();
        }
    }
}

struct Hardware {
    rom: Vec<u8>,
    bios: Vec<u8>,
    iwram: MirrorVec<u8>,
    ewram: MirrorVec<u8>,
    audio: Audio,
    dma: Dma,
    post_boot_flag: u8,
}

impl Hardware {
    pub fn new(rom: Vec<u8>, bios: Vec<u8>) -> Self {
        let title = String::from_utf8_lossy(&rom[0xa0..=0xab]).into_owned();

        info!("Title: {}", title);

        Self {
            rom,
            bios,
            iwram: MirrorVec::new(IWRAM_SIZE),
            ewram: MirrorVec::new(EWRAM_SIZE),
            audio: Audio::new(),
            dma: Dma::new(),
            post_boot_flag: 0,
        }
    }
}

impl DataReader for Hardware {
    type Address = u32;
    type Value = u8;

    fn read(&self, address: u32) -> u8 {
        match address {
            0x0300 => self.post_boot_flag,
            _ => todo!("I/O Register Read: {:08X}", address),
        }
    }
}

impl DataWriter for Hardware {
    fn write(&mut self, address: u32, value: u8) {
        match address {
            0x0300 => {
                self.post_boot_flag = value;
                debug!("Post-Boot Flag: {:02X}", self.post_boot_flag);
            }
            _ => warn!(
                "not yet implemented: I/O Register Write: {:08X} <= {:02X}",
                address, value
            ),
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
                address => self.read_le(address),
            },
            0x05 => todo!("Palette RAM Reads"),
            0x06 => todo!("VRAM Reads"),
            0x07 => todo!("OAM Reads"),
            0x08..=0x0d => self.rom.read_le(address as usize & 0x01ff_ffff),
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
                address => self.write_le(address, value),
            },
            0x05 => warn!("Palette RAM Writes not yet implemented"),
            0x06 => warn!("VRAM Writes not yet implemented"),
            0x07 => warn!("OAM Writes not yet implemented"),
            0x08..=0x0d => panic!("Write to ROM area: {:08X} <= {:08X}", address, value),
            0xe0 => warn!("SRAM Writes not yet implemented"),
            _ => panic!("Unmapped Write: {:08X} <= {:08X}", address, value),
        }
    }
}

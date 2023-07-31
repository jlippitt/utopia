use super::{BiosLoader, System};
use crate::core::arm7tdmi::{Bus, Core};
use crate::util::facade::{DataReader, ReadFacade, Value};
use crate::JoypadState;
use std::error::Error;
use tracing::info;

const WIDTH: usize = 240;
const HEIGHT: usize = 160;
const PIXELS: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];

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
    post_boot_flag: u8,
}

impl Hardware {
    pub fn new(rom: Vec<u8>, bios: Vec<u8>) -> Self {
        let title = String::from_utf8_lossy(&rom[0xa0..=0xab]).into_owned();

        info!("Title: {}", title);

        Self {
            rom,
            bios,
            post_boot_flag: 0,
        }
    }
}

impl DataReader for Hardware {
    type Address = u32;
    type Value = u8;

    fn read(&self, address: Self::Address) -> Self::Value {
        match address {
            0x0300 => self.post_boot_flag,
            _ => todo!("I/O Register Read: {:08X}", address),
        }
    }
}

impl Bus for Hardware {
    fn read<T: Value>(&mut self, address: u32) -> T {
        match address >> 24 {
            0x00 => self.bios.read_le(address as usize),
            0x02 => todo!("EWRAM Reads"),
            0x03 => todo!("IWRAM Reads"),
            0x04 => match address & 0x00ff_ffff {
                0x0000..=0x005f => todo!("LCD Register Reads"),
                0x0060..=0x00af => todo!("Audio Register Reads"),
                0x00b0..=0x00ff => todo!("DMA Register Reads"),
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
}

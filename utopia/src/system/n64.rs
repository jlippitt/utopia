use super::System;
use crate::core::mips::{Bus, Core, State};
use crate::util::facade::{ReadFacade, Value};
use crate::JoypadState;
use rdram::Rdram;
use rsp::{Rsp, DMEM_SIZE};
use std::error::Error;
use tracing::info;

mod header;
mod rdram;
mod rsp;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const PIXELS: [u8; WIDTH * 4 * HEIGHT] = [0; WIDTH * 4 * HEIGHT];

const IPL3_START_ADDRESS: u32 = 0xA4000040;

pub struct N64 {
    core: Core<Hardware>,
}

impl N64 {
    pub fn new(rom_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let header = header::parse(&rom_data);

        info!("Title: {}", header.title);
        info!("Boot Address: {:08X}", header.boot_address);

        let hw = Hardware::new(rom_data);

        let core = Core::new(
            hw,
            State {
                pc: IPL3_START_ADDRESS,
                ..Default::default()
            },
        );

        Ok(N64 { core })
    }
}

impl System for N64 {
    fn width(&self) -> usize {
        // TODO: Support for multiple resolutions
        // (Needs front-end changes!)
        WIDTH
    }

    fn height(&self) -> usize {
        // TODO: Support for multiple resolutions
        // (Needs front-end changes!)
        HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        &PIXELS
    }

    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        // TODO: Timing
        loop {
            self.core.step();
        }
    }
}

struct Hardware {
    rdram: Rdram,
    rsp: Rsp,
    rom: Vec<u8>,
}

impl Hardware {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rdram: Rdram::new(),
            rsp: Rsp::new(&rom[0..DMEM_SIZE]),
            rom,
        }
    }

    fn read_physical<T: Value>(&mut self, address: u32) -> T {
        match address >> 20 {
            0x000..=0x03e => todo!("RDRAM"),
            0x03f => todo!("RDRAM Registers"),
            0x040 => self.rsp.read(address),
            0x041 => todo!("RDP Command Registers"),
            0x042 => todo!("RDP Span Registers"),
            0x043 => todo!("MIPS Interface"),
            0x044 => todo!("Video Interface"),
            0x045 => todo!("Audio Interface"),
            0x046 => todo!("Peripheral Interface"),
            0x047 => self.rdram.interface().read_be(address as usize),
            0x048 => todo!("Serial Interface"),
            0x080..=0x0ff => todo!("SRAM"),
            0x010..=0x1fb => self.rom.read_be(address as usize),
            0x1fc => todo!("Serial Bus"),
            _ => panic!("Read from open bus: {:08X}", address),
        }
    }
}

impl Bus for Hardware {
    fn read<T: Value>(&mut self, address: u32) -> T {
        match address >> 29 {
            4 => self.read_physical(address - 0x8000_0000), // TODO: Cache
            5 => self.read_physical(address - 0xa000_0000),
            _ => todo!("TLB"),
        }
    }
}

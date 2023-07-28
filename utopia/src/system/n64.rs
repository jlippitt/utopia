use super::System;
use crate::core::mips::{Bus, Core, State};
use crate::util::facade::{ReadFacade, Value, WriteFacade};
use crate::JoypadState;
use mips::MipsInterface;
use rdram::Rdram;
use rsp::{Rsp, DMEM_SIZE};
use std::error::Error;
use tracing::info;

mod header;
mod mips;
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

        let mut regs: [u32; 32] = Default::default();

        regs[19] = 0; // $S3
        regs[20] = 1; // $S4
        regs[21] = 0; // $S5
        regs[22] = 0x3f; // $S6
        regs[23] = 0; // $S7
        regs[29] = 0xa4001ff0; // $SP

        let core = Core::new(
            hw,
            State {
                pc: IPL3_START_ADDRESS,
                regs,
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
    mips_interface: MipsInterface,
    rom: Vec<u8>,
}

impl Hardware {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rdram: Rdram::new(),
            rsp: Rsp::new(&rom[0..DMEM_SIZE]),
            mips_interface: MipsInterface::new(),
            rom,
        }
    }

    fn read_physical<T: Value>(&mut self, address: u32) -> T {
        match address >> 20 {
            0x000..=0x03e => todo!("RDRAM Reads"),
            0x03f => todo!("RDRAM Register Reads"),
            0x040 => self.rsp.read(address & 0x000f_ffff),
            0x041 => todo!("RDP Command Register Reads"),
            0x042 => todo!("RDP Span Register Reads"),
            0x043 => self.mips_interface.read_be(address & 0x000f_ffff),
            0x044 => todo!("Video Interface Reads"),
            0x045 => todo!("Audio Interface Reads"),
            0x046 => todo!("Peripheral Interface Reads"),
            0x047 => self.rdram.interface().read_be(address & 0x000f_ffff),
            0x048 => todo!("Serial Interface Reads"),
            0x080..=0x0ff => todo!("SRAM Reads"),
            0x100..=0x1fb => self.rom.read_be((address & 0x0fff_ffff) as usize),
            0x1fc => todo!("Serial Bus Reads"),
            _ => panic!("Read from open bus: {:08X}", address),
        }
    }

    fn write_physical<T: Value>(&mut self, address: u32, value: T) {
        match address >> 20 {
            0x000..=0x03e => todo!("RDRAM Writes"),
            0x03f => todo!("RDRAM Register Writes"),
            0x040 => self.rsp.write(address & 0x000f_ffff, value),
            0x041 => todo!("RDP Command Register Writes"),
            0x042 => todo!("RDP Span Register Writes"),
            0x043 => self.mips_interface.write_be(address & 0x000f_ffff, value),
            0x044 => todo!("Video Interface Writes"),
            0x045 => todo!("Audio Interface Writes"),
            0x046 => todo!("Peripheral Interface Writes"),
            0x047 => self
                .rdram
                .interface_mut()
                .write_be(address & 0x000f_ffff, value),
            0x048 => todo!("Serial Interface Writes"),
            0x080..=0x0ff => todo!("SRAM Writes"),
            0x100..=0x1fb => panic!("Write to ROM area: {:08X}", address),
            0x1fc => todo!("Serial Bus Writes"),
            _ => panic!("Write to open bus: {:08X}", address),
        }
    }
}

impl Bus for Hardware {
    fn read<T: Value>(&mut self, address: u32) -> T {
        match address >> 29 {
            //4 => self.read_physical(address - 0x8000_0000),
            5 => self.read_physical(address - 0xa000_0000),
            _ => todo!("TLB"),
        }
    }

    fn write<T: Value>(&mut self, address: u32, value: T) {
        match address >> 29 {
            //4 => self.write_physical(address - 0x8000_0000, value),
            5 => self.write_physical(address - 0xa000_0000, value),
            _ => todo!("TLB"),
        }
    }
}

use crate::util::facade::{DataReader, DataWriter};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::debug;

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

// TODO: Resolutions other than 320*200
const PIXEL_BUFFER_SIZE: usize = WIDTH * HEIGHT * 4;

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum ColorMode {
    Blank = 0,
    Reserved = 1,
    Color16 = 2,
    Color32 = 3,
}

struct Control {
    color_mode: ColorMode,
}

struct Registers {
    ctrl: Control,
    origin: u32,
    width: u32,
    v_intr: u32,
    v_current: u32,
    v_sync: u32,
    h_sync: u32,
    h_start: u32,
    h_end: u32,
    v_start: u32,
    v_end: u32,
}

pub struct VideoInterface {
    ready: bool,
    cycles: u64,
    line: u32,
    field: bool,
    regs: Registers,
    pixels: Vec<u8>,
}

impl VideoInterface {
    pub fn new() -> Self {
        Self {
            ready: false,
            cycles: 0,
            line: 0,
            field: false,
            regs: Registers {
                ctrl: Control {
                    color_mode: ColorMode::Blank,
                },
                origin: 0,
                width: 0,
                v_intr: 0x3ff,
                v_current: 0,
                v_sync: 0x3ff,
                h_sync: 0x7ff,
                h_start: 0x06c,
                h_end: 0x2ec,
                v_start: 0x025,
                v_end: 0x01f,
            },
            pixels: vec![0; PIXEL_BUFFER_SIZE],
        }
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn start_frame(&mut self) {
        self.ready = false;
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pitch(&self) -> usize {
        WIDTH * 4
    }

    pub fn step(&mut self, cycles: u64) {
        self.cycles += cycles;

        // Runs approximately half as fast as the CPU
        let cycles_per_line = (self.regs.h_sync as u64) << 1;

        if self.cycles >= cycles_per_line {
            self.cycles -= cycles_per_line;
            self.line += 1;

            // VCurrent & VSync are given in half lines, with the low bit
            // representing the field in interlace mode
            self.regs.v_current = (self.line << 1) | (self.field as u32);

            // (TODO: Interlace mode)
            if self.regs.v_current >= self.regs.v_sync {
                self.line = 0;
                self.field = !self.field;
                self.regs.v_current = self.field as u32;
                self.ready = true;
                debug!("Field: {}", self.field as u32);
            }

            debug!("Line: {}", self.line);
        }
    }

    pub fn update_pixel_buffer(&mut self, rdram: &[u8]) {
        match self.regs.ctrl.color_mode {
            ColorMode::Color32 => {
                let start = self.regs.origin as usize;
                self.pixels
                    .copy_from_slice(&rdram[start..(start + PIXEL_BUFFER_SIZE)]);
            }
            ColorMode::Color16 => todo!("16-bit color"),
            ColorMode::Reserved => panic!("Using 'reserved' color mode"),
            ColorMode::Blank => self.pixels.fill(0),
        }
    }
}

impl DataReader for VideoInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            0x04 => self.regs.origin,
            0x08 => self.regs.width,
            0x0c => self.regs.v_intr,
            0x10 => self.regs.v_current,
            0x18 => self.regs.v_sync,
            0x1c => self.regs.h_sync,
            _ => unimplemented!("Video Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for VideoInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                // VI_CTRL: TODO
                self.regs.ctrl.color_mode = ColorMode::from_u32(value & 3).unwrap();
                debug!("VI_CTRL Color Mode: {:?}", self.regs.ctrl.color_mode);
            }
            0x04 => {
                self.regs.origin = value & 0x00ff_ffff;
                debug!("VI_ORIGIN: {:08X}", self.regs.origin);
            }
            0x08 => {
                self.regs.width = value & 0x0fff;
                debug!("VI_WIDTH: {}", self.regs.width);
            }
            0x0c => {
                self.regs.v_intr = value & 0x3ff;
                debug!("VI_V_INTR: {}", self.regs.v_intr);
            }
            0x10 => {
                self.regs.v_current = value & 0x3ff;
                debug!("VI_V_CURRENT: {}", self.regs.v_current);
                // TODO: Clear interrupt
            }
            0x14 => {
                // VI_BURST: Ignore for now
            }
            0x18 => {
                self.regs.v_sync = value & 0x3ff;
                debug!("VI_V_SYNC: {}", self.regs.v_sync);
            }
            0x1c => {
                self.regs.h_sync = value & 0x7ff;
                debug!("VI_H_SYNC: {}", self.regs.h_sync);
                // TODO: Leap (PAL only)
            }
            0x20 => {
                // VI_H_SYNC_LEAP: Ignore for now
            }
            0x24 => {
                self.regs.h_start = (value >> 16) & 0x3ff;
                self.regs.h_end = value & 0x3ff;
                debug!("VI_H_START: {}", self.regs.h_start);
                debug!("VI_H_END: {}", self.regs.h_end);
            }
            0x28 => {
                self.regs.v_start = (value >> 16) & 0x3ff;
                self.regs.v_end = value & 0x3ff;
                debug!("VI_V_START: {}", self.regs.v_start);
                debug!("VI_V_END: {}", self.regs.v_end);
            }
            0x2c => {
                // VI_V_BURST: Ignore for now
            }
            0x30 => {
                // VI_X_SCALE: Ignore for now
            }
            0x34 => {
                // VI_Y_SCALE: Ignore for now
            }
            _ => unimplemented!("Video Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

use super::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::facade::{DataReader, DataWriter};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::debug;

const DEFAULT_WIDTH: u32 = 320;
const DEFAULT_HEIGHT: u32 = 474;

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum ColorMode {
    Blank = 0,
    Reserved = 1,
    Color16 = 2,
    Color32 = 3,
}

struct Control {
    color_mode: ColorMode,
    interlace: bool,
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
    x_offset: u32,
    x_scale: u32,
    y_offset: u32,
    y_scale: u32,
}

pub struct VideoInterface {
    ready: bool,
    cycles: u64,
    line: u32,
    field: bool,
    interrupt: RcpInterrupt,
    regs: Registers,
    pixels: Vec<u8>,
}

impl VideoInterface {
    pub fn new(interrupt: RcpInterrupt) -> Self {
        Self {
            ready: false,
            cycles: 0,
            line: 0,
            field: false,
            interrupt,
            regs: Registers {
                ctrl: Control {
                    color_mode: ColorMode::Blank,
                    interlace: false,
                },
                origin: 0,
                width: 0,
                v_intr: 0x3ff,
                v_current: 0,
                v_sync: 0x3ff,
                h_sync: 0x7ff,
                h_start: 0x06c,
                h_end: 0x06c + DEFAULT_WIDTH,
                v_start: 0x025,
                v_end: 0x025 + DEFAULT_HEIGHT,
                x_offset: 0,
                x_scale: 1024,
                y_offset: 0,
                y_scale: 1024,
            },
            pixels: vec![0; DEFAULT_WIDTH as usize * (DEFAULT_HEIGHT / 2) as usize * 4],
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
        ((self.regs.h_end - self.regs.h_start) as usize * 4 * self.regs.x_scale as usize) / 1024
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

            if self.regs.v_current >= self.regs.v_sync {
                self.line = 0;
                self.field ^= self.regs.ctrl.interlace;
                self.regs.v_current = self.field as u32;
                self.ready = true;
            }

            debug!("Line: {} ({})", self.line, self.field as u32);

            if self.regs.v_current == self.regs.v_intr {
                self.interrupt.raise(RcpIntType::VI);
            }
        }
    }

    pub fn update_pixel_buffer(&mut self, rdram: &[u8]) {
        let pitch = self.pitch();

        let height = ((((self.regs.v_end - self.regs.v_start) as usize) >> 1)
            * self.regs.y_scale as usize)
            / 1024;

        let pixel_buffer_size = pitch * height;

        if pixel_buffer_size != self.pixels.len() {
            self.pixels.resize(pixel_buffer_size, 0);
        }

        match self.regs.ctrl.color_mode {
            ColorMode::Color32 => {
                let mut read_pos = self.regs.origin as usize;
                let mut write_pos = 0;

                while write_pos < pixel_buffer_size {
                    self.pixels[write_pos..(write_pos + pitch)]
                        .copy_from_slice(&rdram[read_pos..(read_pos + pitch)]);

                    read_pos += self.regs.width as usize * 4;
                    write_pos += pitch;
                }
            }
            ColorMode::Color16 => {
                let mut read_pos = self.regs.origin as usize;
                let mut write_pos = 0;

                while write_pos < pixel_buffer_size {
                    let iter =
                        rdram[read_pos..(read_pos + pitch / 2)]
                            .chunks(2)
                            .flat_map(|bytes| {
                                let color = u16::from_be_bytes([bytes[0], bytes[1]]);
                                let red = color as u8 & 31;
                                let green = (color >> 5) as u8 & 31;
                                let blue = (color >> 10) as u8 & 31;
                                [blue << 3, green << 3, red << 3, 0]
                            });

                    self.pixels.splice(write_pos..(write_pos + pitch), iter);

                    read_pos += self.regs.width as usize * 2;
                    write_pos += pitch;
                }
            }
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
                self.regs.ctrl.interlace = (value & 0x40) != 0;
                self.regs.ctrl.color_mode = ColorMode::from_u32(value & 3).unwrap();
                debug!("VI_CTRL Interlace: {}", self.regs.ctrl.interlace);
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
                // TODO: Can writing here trigger an immediate interrupt if it equals V_CURRENT?
            }
            0x10 => {
                self.regs.v_current = value & 0x3ff;
                debug!("VI_V_CURRENT: {}", self.regs.v_current);
                self.interrupt.clear(RcpIntType::VI);
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
                self.regs.x_offset = (value >> 16) & 0x0fff;
                self.regs.x_scale = value & 0x0fff;
                debug!("VI_X_OFFSET: {}", self.regs.x_offset);
                debug!("VI_X_SCALE: {}", self.regs.x_scale);
            }
            0x34 => {
                self.regs.y_offset = (value >> 16) & 0x0fff;
                self.regs.y_scale = value & 0x0fff;
                debug!("VI_Y_OFFSET: {}", self.regs.y_offset);
                debug!("VI_Y_SCALE: {}", self.regs.y_scale);
            }
            _ => unimplemented!("Video Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}

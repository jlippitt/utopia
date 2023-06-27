use crate::system::nes::cartridge::Cartridge;

use super::Ppu;
use tracing::debug;

const ATTR_SHIFT: [u32; 4] = [0, 0x5555, 0xaaaa, 0xffff];

pub struct RenderState {
    address: u16,
    name: u16,
    attr_latch: u8,
    chr_latch: u8,
    chr_shift_low: u32,
    chr_shift_high: u32,
    attr_shift: u32,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            address: 0,
            name: 0,
            attr_latch: 0,
            chr_latch: 0,
            chr_shift_low: 0,
            chr_shift_high: 0,
            attr_shift: 0,
        }
    }
}

impl Ppu {
    pub(super) fn draw_pixel(&mut self) {
        let mut color = self.palette.color(0);

        if self.dot >= self.mask.bg_start {
            let shift = 15 - self.regs.x;
            let low = (self.render.chr_shift_low >> shift) & 0x01;
            let high = (self.render.chr_shift_high >> shift) & 0x02;
            let attr = (self.render.attr_shift >> (shift << 1)) & 0x03;
            let index = (attr << 2) | high | low;
            color = self.palette.color(index as usize);
        }

        self.screen.draw(color);
    }

    pub(super) fn load_bg_tiles(&mut self, cartridge: &Cartridge) {
        self.render.chr_shift_low <<= 1;
        self.render.chr_shift_high <<= 1;
        self.render.attr_shift <<= 2;

        match self.dot & 7 {
            0 => {
                self.render.address = 0x2000 | (self.regs.v & 0x0fff);
            }
            1 => {
                let value = cartridge.read_name(self.render.address);
                self.render.name = (value as u16) << 4;
            }
            2 => {
                self.render.address = 0x23c0
                    | (self.regs.v & 0x0c00)
                    | ((self.regs.v >> 4) & 0x38)
                    | ((self.regs.v >> 2) & 0x07);
            }
            3 => {
                let value = cartridge.read_name(self.render.address);
                let shift = ((self.regs.v & 0x40) >> 4) | (self.regs.v & 0x02);
                self.render.attr_latch = (value >> shift) & 0x03;
            }
            4 => {
                self.render.address =
                    self.control.bg_chr_offset | self.render.name | (self.regs.v >> 12);
            }
            5 => {
                self.render.chr_latch = cartridge.read_chr(self.render.address);
            }
            6 => {
                self.render.address =
                    self.control.bg_chr_offset | self.render.name | (self.regs.v >> 12) | 0x08;
            }
            7 => {
                let value = cartridge.read_chr(self.render.address);

                // TODO: This is actually meant to be one cycle later, but that makes timing more complex
                self.render.chr_shift_low =
                    (self.render.chr_shift_low & 0xff00) | (self.render.chr_latch as u32);

                self.render.chr_shift_high =
                    (self.render.chr_shift_high & 0x0001_fe00) | ((value as u32) << 1);

                self.render.attr_shift = (self.render.attr_shift & 0xffff_0000)
                    | ATTR_SHIFT[self.render.attr_latch as usize];

                self.increment_horizontal();
            }
            _ => unreachable!(),
        }
    }

    pub(super) fn copy_horizontal(&mut self) {
        self.regs.v = (self.regs.v & 0x7be0) | (self.regs.t & 0x041f);
        debug!("PPU VRAM Address (Copy Horizontal): {:04X}", self.regs.v);
    }

    pub(super) fn copy_vertical(&mut self) {
        self.regs.v = (self.regs.v & 0x041f) | (self.regs.t & 0x7be0);
        debug!("PPU VRAM Address (Copy Vertical): {:04X}", self.regs.v);
    }

    pub(super) fn increment_horizontal(&mut self) {
        if self.regs.v & 0x1f == 0x1f {
            self.regs.v &= !0x1f;
            self.regs.v ^= 0x0400;
        } else {
            self.regs.v += 1;
        }
    }

    pub(super) fn increment_vertical(&mut self) {
        if (self.regs.v & 0x7000) == 0x7000 {
            self.regs.v &= !0x7000;

            match self.regs.v & 0x03e0 {
                0x03e0 => self.regs.v &= !0x03e0,
                0x03a0 => {
                    self.regs.v &= !0x03e0;
                    self.regs.v ^= 0x0800;
                }
                _ => self.regs.v += 0x20,
            }
        } else {
            self.regs.v += 0x1000;
        }

        debug!("PPU VRAM Address (Increment Vertical): {:04X}", self.regs.v);
    }
}

use crate::system::nes::cartridge::Cartridge;

use super::Ppu;
use tracing::debug;

pub struct RenderState {
    address: u16,
    name: u16,
    attr: u8,
    chr_low: u8,
    chr_high: u8,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            address: 0,
            name: 0,
            attr: 0,
            chr_low: 0,
            chr_high: 0,
        }
    }
}

impl Ppu {
    pub(super) fn load_bg_tiles(&mut self, cartridge: &Cartridge) {
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
                let shift = ((self.regs.v & 0x20) >> 4) | (self.regs.v & 0x01);
                self.render.attr = (value >> shift) & 0x03;
            }
            4 => {
                self.render.address =
                    self.control.bg_chr_offset | self.render.name | (self.regs.v >> 12);
            }
            5 => {
                self.render.chr_low = cartridge.read_chr(self.render.address);
            }
            6 => {
                self.render.address =
                    self.control.bg_chr_offset | self.render.name | (self.regs.v >> 12) | 0x08;
            }
            7 => {
                self.render.chr_high = cartridge.read_chr(self.render.address);

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

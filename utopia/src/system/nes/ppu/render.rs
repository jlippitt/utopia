use super::Ppu;
use tracing::debug;

impl Ppu {
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

        debug!(
            "PPU VRAM Address (Increment Horizontal): {:04X}",
            self.regs.v
        );
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

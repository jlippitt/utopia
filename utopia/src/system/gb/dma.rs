use crate::Mapped;
use tracing::debug;

pub struct Dma {
    src_address: u16,
    dst_address: u16,
    len: u8,
    hblank_mode: bool,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            src_address: 0,
            dst_address: 0,
            len: 0,
            hblank_mode: false,
        }
    }

    pub fn is_hblank_mode(&self) -> bool {
        self.hblank_mode
    }

    pub fn read(&self, address: u8) -> u8 {
        match address {
            0x51 => (self.src_address >> 8) as u8,
            0x52 => self.src_address as u8,
            0x53 => (self.dst_address >> 8) as u8,
            0x54 => self.dst_address as u8,
            0x55 => self.len,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u8, value: u8) -> bool {
        match address {
            0x51 => {
                self.src_address = (self.src_address & 0xff) | ((value as u16) << 8);
                debug!("DMA Source Address: {:04X}", self.src_address);
            }
            0x52 => {
                self.src_address = (self.src_address & 0xff00) | (value as u16 & 0xf0);
                debug!("DMA Source Address: {:04X}", self.src_address);
            }
            0x53 => {
                self.dst_address = (self.dst_address & 0xff) | ((value as u16 & 0x1f) << 8);
                debug!("DMA Destination Address: {:04X}", self.dst_address);
            }
            0x54 => {
                self.dst_address = (self.dst_address & 0xff00) | (value as u16 & 0xf0);
                debug!("DMA Destination Address: {:04X}", self.dst_address);
            }
            0x55 => {
                let prev_hblank_mode = self.hblank_mode;
                self.hblank_mode = (value & 0x80) != 0;

                debug!("DMA HBlank Mode: {}", self.hblank_mode);

                if !self.hblank_mode && prev_hblank_mode {
                    self.len |= 0x80;
                    return false;
                }

                self.len = value & 0x7f;

                debug!(
                    "DMA Length: {} ({})",
                    self.len,
                    (self.len as usize + 1) << 4
                );

                return !self.hblank_mode;
            }
            _ => unreachable!(),
        }

        false
    }
}

impl<T: Mapped> super::Hardware<T> {
    pub fn transfer_vram_dma(&mut self) {
        debug!("DMA Transfer Begin");

        loop {
            for byte in 0..16 {
                let value = self.read_normal(self.dma.src_address);

                debug!(
                    "DMA Write: {:04X} <= {:02X} <= {:04X}",
                    0x8000 + self.dma.dst_address,
                    self.dma.src_address,
                    value
                );

                self.ppu.write_vram(self.dma.dst_address, value);

                self.dma.src_address = self.dma.src_address.wrapping_add(1);
                self.dma.dst_address = self.dma.dst_address.wrapping_add(1) & 0x1fff;

                if (byte & 7) == 7 {
                    self.step();
                }
            }

            self.dma.len = self.dma.len.wrapping_sub(1);

            let done = self.dma.len != 0xff;

            if self.dma.hblank_mode {
                if done {
                    self.dma.hblank_mode = false;
                }

                break;
            } else if done {
                break;
            }
        }

        debug!("DMA Transfer End");
    }
}

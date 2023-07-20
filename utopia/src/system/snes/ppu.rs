use background::BackgroundLayer;
use cgram::Cgram;
use tracing::warn;
use vram::Vram;

mod background;
mod cgram;
mod vram;

pub struct Ppu {
    vram: Vram,
    cgram: Cgram,
    bg: [BackgroundLayer; 4],
    scroll_regs: (u8, u8),
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: Vram::new(),
            cgram: Cgram::new(),
            bg: [
                BackgroundLayer::new(1),
                BackgroundLayer::new(2),
                BackgroundLayer::new(3),
                BackgroundLayer::new(4),
            ],
            scroll_regs: (0, 0),
        }
    }

    pub fn read(&mut self, address: u8) -> u8 {
        match address {
            _ => panic!("Unmapped PPU read: {:02X}", address),
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x07 => self.bg[0].set_tile_map(value),
            0x08 => self.bg[1].set_tile_map(value),
            0x09 => self.bg[2].set_tile_map(value),
            0x0a => self.bg[3].set_tile_map(value),
            0x0b => {
                self.bg[0].set_chr_map(value & 0x0f);
                self.bg[1].set_chr_map(value >> 4);
            }
            0x0c => {
                self.bg[2].set_chr_map(value & 0x0f);
                self.bg[3].set_chr_map(value >> 4);
            }
            0x0d => self.bg[0].set_scroll_x(&mut self.scroll_regs, value),
            0x0e => self.bg[0].set_scroll_y(&mut self.scroll_regs, value),
            0x0f => self.bg[1].set_scroll_x(&mut self.scroll_regs, value),
            0x10 => self.bg[1].set_scroll_y(&mut self.scroll_regs, value),
            0x11 => self.bg[2].set_scroll_x(&mut self.scroll_regs, value),
            0x12 => self.bg[2].set_scroll_y(&mut self.scroll_regs, value),
            0x13 => self.bg[3].set_scroll_x(&mut self.scroll_regs, value),
            0x14 => self.bg[3].set_scroll_y(&mut self.scroll_regs, value),
            0x15 => self.vram.set_control(value),
            0x16 => self.vram.set_address_low(value),
            0x17 => self.vram.set_address_high(value),
            0x18 => self.vram.write_low(value),
            0x19 => self.vram.write_high(value),
            0x21 => self.cgram.set_address(value),
            0x22 => self.cgram.write(value),
            _ => warn!("Unmapped PPU write: {:02X} <= {:02X}", address, value),
        }
    }

    pub fn draw_line(&mut self, line: u16) {
        // TODO: Video modes
        self.draw_bg::<0>(0, 4, 3, line);
    }
}

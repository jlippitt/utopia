use super::interrupt::{Interrupt, InterruptType};
use super::vce::Color;
use background::BackgroundLayer;
use bitflags::bitflags;
use screen::Screen;
use sprite::SpriteLayer;
use tracing::{debug, warn};
use vram::Vram;

mod background;
mod screen;
mod sprite;
mod vram;

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct VdcInterrupt: u8 {
        const SPRITE_COLLISION = 0x01;
        const SPRITE_OVERFLOW = 0x02;
        const SCANLINE = 0x04;
        const VBLANK = 0x08;
    }
}

pub struct Vdc {
    reg_index: u8,
    interrupt_enable: VdcInterrupt,
    interrupt_active: VdcInterrupt,
    scanline_match: u16,
    vram: Vram,
    bg: BackgroundLayer,
    sprite: SpriteLayer,
    line_buffer: Vec<Color>,
    screen: Screen,
    interrupt: Interrupt,
}

impl Vdc {
    pub const DEFAULT_WIDTH: u16 = Screen::DEFAULT_WIDTH;
    pub const DEFAULT_HEIGHT: u16 = Screen::DEFAULT_HEIGHT;

    pub fn new(interrupt: Interrupt) -> Self {
        Self {
            reg_index: 0,
            interrupt_enable: VdcInterrupt::empty(),
            interrupt_active: VdcInterrupt::empty(),
            scanline_match: 0,
            vram: Vram::new(),
            bg: BackgroundLayer::new(),
            sprite: SpriteLayer::new(),
            line_buffer: vec![Color::new(); Self::DEFAULT_WIDTH as usize],
            screen: Screen::new(),
            interrupt,
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.screen.pixels()
    }

    pub fn display_width(&self) -> u16 {
        self.screen.width()
    }

    pub fn display_height(&self) -> u16 {
        self.screen.height()
    }

    pub fn scanline_match(&self) -> u16 {
        self.scanline_match
    }

    pub fn read(&mut self, address: u16, _prev_value: u8) -> u8 {
        match address & 3 {
            0 => {
                let mut status = self.interrupt_active.bits();

                // VBlank bit is annoyingly moved to bit 5
                status = (status & 0x07) | ((status & 0x08) << 2);

                // TODO: DMA status

                self.interrupt_active = VdcInterrupt::empty();
                debug!("VDC Interrupts Cleared");
                self.interrupt.clear(InterruptType::Irq1);

                status
            }
            _ => unimplemented!("VDC Read: {:04X}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => self.reg_index = value & 0x1f,
            2 => self.write_register(false, value),
            3 => self.write_register(true, value),
            _ => warn!("Unimplemented VDC Write: {:04X} <= {:02X}", address, value),
        }
    }

    pub fn on_frame_start(&mut self) {
        self.screen.reset();
    }

    pub fn on_vblank_start(&mut self) {
        self.sprite.transfer_dma(&self.vram);
        self.raise_interrupt(VdcInterrupt::VBLANK);
    }

    pub fn on_scanline_match(&mut self) {
        self.raise_interrupt(VdcInterrupt::SCANLINE);
    }

    pub fn render_line(&mut self, palette: &[Color], line: u16) {
        self.line_buffer.clear();
        self.line_buffer
            .resize(self.screen.width() as usize, palette[0]);

        self.bg
            .render_line(&mut self.line_buffer, &self.vram, palette, line);

        self.screen.draw_line(&self.line_buffer);
    }

    fn write_register(&mut self, msb: bool, value: u8) {
        match self.reg_index {
            0x00 => self.vram.set_write_address(msb, value),
            0x02 => self.vram.write(msb, value),
            0x05 => {
                // TODO: Other settings
                if msb {
                    self.vram.set_increment_amount(match (value >> 3) & 3 {
                        0 => 1,
                        1 => 32,
                        2 => 64,
                        3 => 128,
                        _ => unreachable!(),
                    });
                } else {
                    self.bg.set_enabled((value & 0x80) != 0);
                    self.sprite.set_enabled((value & 0x40) != 0);
                    self.interrupt_enable = VdcInterrupt::from_bits_retain(value & 0x0f);
                    debug!("VDC Interrupt Enable: {:?}", self.interrupt_enable);
                }
            }
            0x06 => {
                self.scanline_match = if msb {
                    (self.scanline_match & 0xff) | ((value as u16 & 0x03) << 8)
                } else {
                    (self.scanline_match & 0xff00) | value as u16
                };
                debug!("VDC Scanline Match: {}", self.scanline_match,);
            }
            0x07 => self.bg.set_scroll_x(msb, value),
            0x08 => self.bg.set_scroll_y(msb, value),
            0x09 => {
                if !msb {
                    self.bg.set_tile_map_size(value);
                }
            }
            0x0b => self.screen.set_width(msb, value),
            0x0d => self.screen.set_height(msb, value),
            0x0f => {
                if !msb {
                    // TODO: Sprite DMA interrupt
                    // TODO: Other flags
                    self.sprite.set_dma_repeat((value & 0x10) != 0);
                }
            }
            0x13 => self.sprite.set_table_address(msb, value),
            _ => warn!(
                "Unimplemented VDC Register Write: {:02X} <= {:04X}",
                self.reg_index, value
            ),
        }
    }

    fn raise_interrupt(&mut self, int_type: VdcInterrupt) {
        if !self.interrupt_enable.contains(int_type) {
            return;
        }

        self.interrupt_active |= int_type;
        debug!("VDC Interrupt Raised: {:?}", int_type);
        self.interrupt.raise(InterruptType::Irq1);
    }
}

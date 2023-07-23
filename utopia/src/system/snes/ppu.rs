pub use screen::{HEIGHT, WIDTH};

use super::clock::Clock;
use background::BackgroundLayer;
use buffer::{Pixel, PixelBuffer, TileBuffer, LAYER_BACKDROP, PIXEL_BUFFER_SIZE, TILE_BUFFER_SIZE};
use cgram::Cgram;
use color_math::ColorMath;
use mode7::Mode7Settings;
use oam::Oam;
use screen::Screen;
use toggle::Toggle;
use tracing::{debug, warn};
use vram::Vram;
use window::{Window, WindowMask};

mod background;
mod buffer;
mod cgram;
mod color_math;
mod mode7;
mod oam;
mod screen;
mod toggle;
mod vram;
mod window;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Bg3Priority {
    Low = 2,
    High = 5,
}

pub struct Ppu {
    force_blank: bool,
    bg_mode: u8,
    bg3_priority: Bg3Priority,
    enabled: [Toggle; 4],
    bg: [BackgroundLayer; 4],
    mode7: Mode7Settings,
    window: [Window; 2],
    window_enabled: [Toggle; 4],
    window_mask: [WindowMask; 6],
    color_math: ColorMath,
    screen: Screen,
    scroll_regs: (u8, u8),
    tiles: TileBuffer,
    pixels: [PixelBuffer; 2],
    vram: Vram,
    cgram: Cgram,
    oam: Oam,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            force_blank: true,
            bg_mode: 0,
            bg3_priority: Bg3Priority::Low,
            enabled: [
                Toggle::new("BG1"),
                Toggle::new("BG2"),
                Toggle::new("BG3"),
                Toggle::new("BG4"),
            ],
            bg: [
                BackgroundLayer::new("BG1"),
                BackgroundLayer::new("BG2"),
                BackgroundLayer::new("BG3"),
                BackgroundLayer::new("BG4"),
            ],
            mode7: Mode7Settings::new(),
            window: [Window::new("W1"), Window::new("W2")],
            window_enabled: [
                Toggle::new("BG1 Window"),
                Toggle::new("BG2 Window"),
                Toggle::new("BG3 Window"),
                Toggle::new("BG4 Window"),
            ],
            window_mask: [
                WindowMask::new("BG1 Mask"),
                WindowMask::new("BG2 Mask"),
                WindowMask::new("BG3 Mask"),
                WindowMask::new("BG4 Mask"),
                WindowMask::new("OBJ Mask"),
                WindowMask::new("Color Math Mask"),
            ],
            color_math: ColorMath::new(),
            screen: Screen::new(),
            scroll_regs: (0, 0),
            tiles: [Default::default(); TILE_BUFFER_SIZE],
            pixels: [
                [Default::default(); PIXEL_BUFFER_SIZE],
                [Default::default(); PIXEL_BUFFER_SIZE],
            ],
            vram: Vram::new(),
            cgram: Cgram::new(),
            oam: Oam::new(),
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.screen.output()
    }

    pub fn read(&mut self, clock: &Clock, address: u8) -> u8 {
        match address {
            0x34 => self.mode7.multiply() as u8,
            0x35 => (self.mode7.multiply() >> 8) as u8,
            0x36 => (self.mode7.multiply() >> 16) as u8,
            0x3f => {
                // TODO: PPU open bus
                let mut value = 0x03;
                value |= if clock.odd_frame() { 0x80 } else { 0 };
                // TODO: External latch flag
                value
            }
            _ => panic!("Unmapped PPU read: {:02X}", address),
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x00 => {
                self.force_blank = (value & 0x80) != 0;
                debug!("Force Blank: {}", self.force_blank);
                self.screen.set_brightness(value & 0x0f);
            }
            0x02 => self.oam.set_address_low(value),
            0x03 => self.oam.set_address_high(value),
            0x04 => self.oam.write(value),
            0x05 => {
                // TODO: 16x16 tiles

                self.bg_mode = value & 0x07;

                self.bg3_priority = if (value & 0x08) != 0 {
                    Bg3Priority::High
                } else {
                    Bg3Priority::Low
                };

                debug!("BG Mode: {}", self.bg_mode);
                debug!("BG3 Priority: {:?}", self.bg3_priority);
            }
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
            0x0d => {
                self.bg[0].set_scroll_x(&mut self.scroll_regs, value);
                //self.mode7.set_scroll_x(value);
            }
            0x0e => {
                self.bg[0].set_scroll_y(&mut self.scroll_regs, value);
                //self.mode7.set_scroll_y(value);
            }
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
            //0x1a => self.mode7.set_control(value),
            0x1b => self.mode7.set_matrix_a(value),
            0x1c => self.mode7.set_matrix_b(value),
            0x1d => self.mode7.set_matrix_c(value),
            0x1e => self.mode7.set_matrix_d(value),
            //0x1f => self.mode7.set_center_x(value),
            //0x20 => self.mode7.set_center_y(value),
            0x21 => self.cgram.set_address(value),
            0x22 => self.cgram.write(value),
            0x23 => {
                self.window_mask[0].set_control(value & 0x0f);
                self.window_mask[1].set_control(value >> 4);
            }
            0x24 => {
                self.window_mask[2].set_control(value & 0x0f);
                self.window_mask[3].set_control(value >> 4);
            }
            0x25 => {
                self.window_mask[4].set_control(value & 0x0f);
                self.window_mask[5].set_control(value >> 4);
            }
            0x26 => self.update_window(|ppu| ppu.window[0].set_left(value)),
            0x27 => self.update_window(|ppu| ppu.window[0].set_right(value)),
            0x28 => self.update_window(|ppu| ppu.window[1].set_left(value)),
            0x29 => self.update_window(|ppu| ppu.window[1].set_right(value)),
            0x2a => {
                self.window_mask[0].set_operator(value & 0x03);
                self.window_mask[1].set_operator((value >> 2) & 0x03);
                self.window_mask[2].set_operator((value >> 4) & 0x03);
                self.window_mask[3].set_operator(value >> 6);
            }
            0x2b => {
                self.window_mask[4].set_operator(value & 0x03);
                self.window_mask[5].set_operator((value >> 2) & 0x03);
            }
            0x2c => {
                self.enabled[0].set(0, (value & 0x01) != 0);
                self.enabled[1].set(0, (value & 0x02) != 0);
                self.enabled[2].set(0, (value & 0x04) != 0);
                self.enabled[3].set(0, (value & 0x08) != 0);
                // TODO: OBJ
            }
            0x2d => {
                self.enabled[0].set(1, (value & 0x01) != 0);
                self.enabled[1].set(1, (value & 0x02) != 0);
                self.enabled[2].set(1, (value & 0x04) != 0);
                self.enabled[3].set(1, (value & 0x08) != 0);
                // TODO: OBJ
            }
            0x2e => {
                self.window_enabled[0].set(0, (value & 0x01) != 0);
                self.window_enabled[1].set(0, (value & 0x02) != 0);
                self.window_enabled[2].set(0, (value & 0x04) != 0);
                self.window_enabled[3].set(0, (value & 0x08) != 0);
                // TODO: OBJ
            }
            0x2f => {
                self.window_enabled[0].set(1, (value & 0x01) != 0);
                self.window_enabled[1].set(1, (value & 0x02) != 0);
                self.window_enabled[2].set(1, (value & 0x04) != 0);
                self.window_enabled[3].set(1, (value & 0x08) != 0);
                // TODO: OBJ
            }
            0x30 => {
                // TODO: Direct Color Mode
                self.color_math.set_control(value);
            }
            0x31 => self.color_math.set_operator(value),
            0x32 => self.color_math.set_fixed_color(value),
            _ => warn!("Unmapped PPU write: {:02X} <= {:02X}", address, value),
        }
    }

    pub fn on_frame_start(&mut self) {
        self.screen.reset();
    }

    pub fn on_vblank_start(&mut self) {
        if !self.force_blank {
            self.oam.reload_internal_address();
        }
    }

    pub fn draw_line(&mut self, line: u16) {
        if self.force_blank {
            self.screen.force_blank();
            return;
        }

        let backdrop_color = self.cgram.color(0);

        for index in [0, 1] {
            self.pixels[index].fill(Pixel {
                color: backdrop_color,
                priority: 0,
                layer: LAYER_BACKDROP,
            });
        }

        match self.bg_mode {
            0 => {
                self.draw_bg::<0>(0, 4, 3, line);
                self.draw_bg::<0>(1, 4, 3, line);
                self.draw_bg::<0>(2, 2, 1, line);
                self.draw_bg::<0>(3, 2, 1, line);
            }
            1 => {
                self.draw_bg::<1>(0, 4, 3, line);
                self.draw_bg::<1>(1, 4, 3, line);
                self.draw_bg::<0>(2, self.bg3_priority as u8, 1, line);
            }
            3 => {
                self.draw_bg::<2>(0, 4, 2, line);
                self.draw_bg::<1>(1, 3, 1, line);
            }
            _ => panic!("Mode {} not yet implemented", self.bg_mode),
        }

        self.apply_color_math();

        self.screen.draw_lo_res(&self.pixels[0]);
    }

    fn update_window(&mut self, callback: impl Fn(&mut Self) -> bool) {
        if callback(self) {
            for mask in &mut self.window_mask {
                mask.mark_as_dirty();
            }
        }
    }
}

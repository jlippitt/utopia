pub use screen::{HEIGHT, WIDTH};

use super::clock::Clock;
use background::BackgroundLayer;
use bitflags::bitflags;
use buffer::{
    OffsetBuffer, Pixel, PixelBuffer, TileBuffer, LAYER_BACKDROP, OFFSET_BUFFER_SIZE,
    PIXEL_BUFFER_SIZE, TILE_BUFFER_SIZE,
};
use cgram::Cgram;
use color_math::ColorMath;
use latch::Latch;
use mode7::Mode7Settings;
use oam::Oam;
use object::ObjectLayer;
use screen::Screen;
use std::ops::RangeInclusive;
use toggle::Toggle;
use tracing::{debug, warn};
use vram::Vram;
use window::{Window, WindowMask};

mod background;
mod buffer;
mod cgram;
mod color_math;
mod latch;
mod mode7;
mod oam;
mod object;
mod screen;
mod toggle;
mod vram;
mod window;

const VISIBLE_RANGE_NORMAL: RangeInclusive<u16> = 1..=224;
const VISIBLE_RANGE_OVERSCAN: RangeInclusive<u16> = 9..=232;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Bg3Priority {
    Low = 2,
    High = 5,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct HiRes: u8 {
        const BG_MODE = 0x01;
        const PSEUDO_HI_RES = 0x02;
    }
}

pub struct Ppu {
    visible_range: RangeInclusive<u16>,
    force_blank: bool,
    bg_mode: u8,
    bg3_priority: Bg3Priority,
    hi_res: HiRes,
    enabled: [Toggle; 5],
    bg: [BackgroundLayer; 4],
    obj: ObjectLayer,
    mode7: Mode7Settings,
    window: [Window; 2],
    window_enabled: [Toggle; 5],
    window_mask: [WindowMask; 6],
    color_math: ColorMath,
    screen: Screen,
    offsets: OffsetBuffer,
    tiles: TileBuffer,
    pixels: [PixelBuffer; 2],
    vram: Vram,
    cgram: Cgram,
    oam: Oam,
    scroll_regs: (u8, u8),
    overscan: bool,
    latch: Latch,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            visible_range: VISIBLE_RANGE_NORMAL,
            force_blank: true,
            bg_mode: 0,
            bg3_priority: Bg3Priority::Low,
            hi_res: HiRes::empty(),
            enabled: [
                Toggle::new("BG1"),
                Toggle::new("BG2"),
                Toggle::new("BG3"),
                Toggle::new("BG4"),
                Toggle::new("OBJ"),
            ],
            bg: [
                BackgroundLayer::new("BG1"),
                BackgroundLayer::new("BG2"),
                BackgroundLayer::new("BG3"),
                BackgroundLayer::new("BG4"),
            ],
            obj: ObjectLayer::new(),
            mode7: Mode7Settings::new(),
            window: [Window::new("W1"), Window::new("W2")],
            window_enabled: [
                Toggle::new("BG1 Window"),
                Toggle::new("BG2 Window"),
                Toggle::new("BG3 Window"),
                Toggle::new("BG4 Window"),
                Toggle::new("OBJ Window"),
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
            offsets: [Default::default(); OFFSET_BUFFER_SIZE],
            tiles: [Default::default(); TILE_BUFFER_SIZE],
            pixels: [
                [Default::default(); PIXEL_BUFFER_SIZE],
                [Default::default(); PIXEL_BUFFER_SIZE],
            ],
            vram: Vram::new(),
            cgram: Cgram::new(),
            oam: Oam::new(),
            scroll_regs: (0, 0),
            overscan: false,
            latch: Latch::new(),
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
            0x37 => {
                self.latch.latch_counters(clock);
                // TODO: PPU Open Bus
                0
            }
            0x38 => self.oam.read(),
            0x39 => self.vram.read_low(),
            0x3a => self.vram.read_high(),
            0x3b => self.cgram.read(),       // TODO: PPU Open Bus
            0x3c => self.latch.counter(0),   // TODO: PPU Open Bus
            0x3d => self.latch.counter(1),   // TODO: PPU Open Bus
            0x3e => self.obj.flags() | 0x01, // TODO: PPU Open Bus
            0x3f => {
                // TODO: PPU open bus
                let mut value = 0x03;
                value |= if clock.odd_frame() { 0x80 } else { 0 };
                value |= if self.latch.poll_status() { 0x40 } else { 0 };
                value
            }
            _ => panic!("Unmapped PPU read: {:02X}", address),
        }
    }

    pub fn write(&mut self, clock: &mut Clock, address: u8, value: u8) {
        match address {
            0x00 => {
                self.force_blank = (value & 0x80) != 0;
                debug!("Force Blank: {}", self.force_blank);
                self.screen.set_brightness(value & 0x0f);
            }
            0x01 => self.obj.set_control(value),
            0x02 => self.oam.set_address_low(value),
            0x03 => self.oam.set_address_high(value),
            0x04 => self.oam.write(value),
            0x05 => {
                self.bg_mode = value & 0x07;

                self.bg3_priority = if (value & 0x08) != 0 {
                    Bg3Priority::High
                } else {
                    Bg3Priority::Low
                };

                self.hi_res
                    .set(HiRes::BG_MODE, self.bg_mode == 5 || self.bg_mode == 6);

                debug!("BG Mode: {}", self.bg_mode);
                debug!("BG3 Priority: {:?}", self.bg3_priority);
                debug!("Hi Res: {:?}", self.hi_res);

                self.bg[0].set_tile_size((value & 0x10) != 0);
                self.bg[1].set_tile_size((value & 0x20) != 0);
                self.bg[2].set_tile_size((value & 0x40) != 0);
                self.bg[3].set_tile_size((value & 0x80) != 0);
            }
            0x06 => {
                let mosaic_size = (value >> 4) + 1;
                self.bg[0].set_mosaic((value & 0x01) != 0, mosaic_size);
                self.bg[1].set_mosaic((value & 0x02) != 0, mosaic_size);
                self.bg[2].set_mosaic((value & 0x04) != 0, mosaic_size);
                self.bg[3].set_mosaic((value & 0x08) != 0, mosaic_size);
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
                self.mode7.set_scroll_x(value);
            }
            0x0e => {
                self.bg[0].set_scroll_y(&mut self.scroll_regs, value);
                self.mode7.set_scroll_y(value);
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
            0x1a => self.mode7.set_control(value),
            0x1b => self.mode7.set_matrix_a(value),
            0x1c => self.mode7.set_matrix_b(value),
            0x1d => self.mode7.set_matrix_c(value),
            0x1e => self.mode7.set_matrix_d(value),
            0x1f => self.mode7.set_center_x(value),
            0x20 => self.mode7.set_center_y(value),
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
                self.enabled[4].set(0, (value & 0x10) != 0);
            }
            0x2d => {
                self.enabled[0].set(1, (value & 0x01) != 0);
                self.enabled[1].set(1, (value & 0x02) != 0);
                self.enabled[2].set(1, (value & 0x04) != 0);
                self.enabled[3].set(1, (value & 0x08) != 0);
                self.enabled[4].set(1, (value & 0x10) != 0);
            }
            0x2e => {
                self.window_enabled[0].set(0, (value & 0x01) != 0);
                self.window_enabled[1].set(0, (value & 0x02) != 0);
                self.window_enabled[2].set(0, (value & 0x04) != 0);
                self.window_enabled[3].set(0, (value & 0x08) != 0);
                self.window_enabled[4].set(0, (value & 0x10) != 0);
            }
            0x2f => {
                self.window_enabled[0].set(1, (value & 0x01) != 0);
                self.window_enabled[1].set(1, (value & 0x02) != 0);
                self.window_enabled[2].set(1, (value & 0x04) != 0);
                self.window_enabled[3].set(1, (value & 0x08) != 0);
                self.window_enabled[4].set(1, (value & 0x10) != 0);
            }
            0x30 => {
                // TODO: Direct Color Mode
                self.color_math.set_control(value);
            }
            0x31 => self.color_math.set_operator(value),
            0x32 => self.color_math.set_fixed_color(value),
            0x33 => {
                clock.set_interlace((value & 0x01) != 0);

                self.obj.set_interlace((value & 0x02) != 0);

                self.overscan = (value & 0x04) != 0;
                debug!("Overscan: {}", self.overscan);

                self.hi_res.set(HiRes::PSEUDO_HI_RES, (value & 0x08) != 0);
                debug!("Hi Res: {:?}", self.hi_res);

                if (value & 0x40) != 0 {
                    todo!("Mode 7 EXTBG");
                }
            }
            _ => warn!("Unmapped PPU write: {:02X} <= {:02X}", address, value),
        }
    }

    pub fn on_frame_start(&mut self, clock: &mut Clock) {
        self.screen.reset();

        self.bg[0].reset_mosaic_counter();
        self.bg[1].reset_mosaic_counter();
        self.bg[2].reset_mosaic_counter();
        self.bg[3].reset_mosaic_counter();

        if !self.force_blank {
            self.obj.clear_flags();
        }

        // We do all this at the start of the frame, instead of when the flag
        // is set, because changing modes in the middle of a frame causes all
        // sorts of problems
        clock.set_overscan(self.overscan);

        self.visible_range = if self.overscan {
            VISIBLE_RANGE_OVERSCAN
        } else {
            VISIBLE_RANGE_NORMAL
        };

        debug!("Visible Range: {:?}", self.visible_range);
    }

    pub fn on_vblank_start(&mut self) {
        if !self.force_blank {
            self.oam.reload_internal_address();
        }
    }

    pub fn set_latch_enabled(&mut self, clock: &Clock, enabled: bool) {
        self.latch.set_enabled(clock, enabled);
    }

    pub fn draw_line(&mut self, line: u16, interlace: bool, odd_frame: bool) {
        if !self.visible_range.contains(&line) {
            // This line is hidden by overscan, so we only need to draw the
            // object layer as this has side effects on register $213E
            if !self.force_blank {
                self.draw_obj(line - 1, odd_frame);
            }
            return;
        }

        if interlace && odd_frame {
            self.screen.skip_line();
        }

        if self.force_blank {
            self.screen.force_blank();
        } else {
            self.draw_layers(line, interlace, odd_frame);
        }

        if !interlace {
            self.screen.duplicate_line();
        } else if !odd_frame {
            self.screen.skip_line();
        }
    }

    fn draw_layers(&mut self, line: u16, interlace: bool, odd_frame: bool) {
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
                self.draw_bg::<0, 0, false>(0, 4, 3, 0, line);
                self.draw_bg::<0, 0, false>(1, 4, 3, 32, line);
                self.draw_bg::<0, 0, false>(2, 2, 1, 64, line);
                self.draw_bg::<0, 0, false>(3, 2, 1, 96, line);
            }
            1 => {
                self.draw_bg::<1, 0, false>(0, 4, 3, 0, line);
                self.draw_bg::<1, 0, false>(1, 4, 3, 0, line);
                self.draw_bg::<0, 0, false>(2, self.bg3_priority as u8, 1, 0, line);
            }
            2 => {
                self.select_bg_offsets::<false>(2);
                self.draw_bg::<1, 0x2000, false>(0, 4, 2, 0, line);
                self.draw_bg::<1, 0x4000, false>(1, 3, 1, 0, line);
            }
            3 => {
                self.draw_bg::<2, 0, false>(0, 4, 2, 0, line);
                self.draw_bg::<1, 0, false>(1, 3, 1, 0, line);
            }
            5 => {
                let line = if interlace {
                    (line << 1) + (odd_frame as u16)
                } else {
                    line
                };

                self.draw_bg::<1, 0, true>(0, 4, 2, 0, line);
                self.draw_bg::<0, 0, true>(1, 3, 1, 0, line);
            }
            7 => {
                self.draw_mode7(0, line);
                // TODO: Mode 7 EXT
            }
            _ => panic!("Mode {} not yet implemented", self.bg_mode),
        }

        self.draw_obj(line - 1, odd_frame);

        self.apply_color_math();

        if self.hi_res.is_empty() {
            self.screen.draw_lo_res(&self.pixels[0]);
        } else {
            self.screen.draw_hi_res(&self.pixels);
        }
    }

    fn update_window(&mut self, callback: impl Fn(&mut Self) -> bool) {
        if callback(self) {
            for mask in &mut self.window_mask {
                mask.mark_as_dirty();
            }
        }
    }
}

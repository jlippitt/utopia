pub use screen::{HEIGHT, WIDTH};

use super::cartridge::Cartridge;
use crate::core::mos6502::{Interrupt, INT_NMI};
use palette::Palette;
use render::RenderState;
use screen::Screen;
use tracing::{debug, warn};

mod palette;
mod render;
mod screen;

const PRE_RENDER_LINE: i32 = -1;
const TOTAL_VISIBLE_LINES: i32 = 240;
const VBLANK_LINE: i32 = 241;
const MAX_LINE_NUMBER: i32 = 261;

struct Registers {
    v: u16,
    t: u16,
    x: u8,
    w: bool,
}

struct Control {
    vram_increment: u16,
    bg_chr_offset: u16,
}

struct Mask {
    render_enabled: bool,
    bg_start: i32,
}

pub struct Ppu {
    ready: bool,
    line: i32,
    dot: i32,
    nmi_occurred: bool,
    nmi_active: bool,
    regs: Registers,
    control: Control,
    mask: Mask,
    render: RenderState,
    palette: Palette,
    screen: Screen,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ready: false,
            line: 0,
            dot: 0,
            nmi_occurred: false,
            nmi_active: false,
            regs: Registers {
                v: 0,
                t: 0,
                x: 0,
                w: false,
            },
            control: Control {
                vram_increment: 1,
                bg_chr_offset: 0,
            },
            mask: Mask {
                render_enabled: false,
                bg_start: 0,
            },
            render: RenderState::new(),
            palette: Palette::new(),
            screen: Screen::new(),
        }
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn start_frame(&mut self) {
        self.ready = false;
    }

    pub fn line(&self) -> i32 {
        self.line
    }

    pub fn dot(&self) -> i32 {
        self.dot
    }

    pub fn pixels(&self) -> &[u8] {
        self.screen.pixels()
    }

    pub fn read(
        &mut self,
        _cartridge: &mut Cartridge,
        interrupt: &mut Interrupt,
        address: u16,
    ) -> u8 {
        match address & 7 {
            2 => {
                // TODO: Open bus
                let mut result: u8 = 0;

                if self.nmi_occurred {
                    result |= 0x80;
                    self.nmi_occurred = false;
                    debug!("NMI Occurred: {}", self.nmi_occurred);
                    *interrupt &= !INT_NMI;
                }

                self.regs.w = false;

                result
            }
            _ => panic!("PPU read {:04X} not yet implemented", address),
        }
    }

    pub fn write(
        &mut self,
        cartridge: &mut Cartridge,
        interrupt: &mut Interrupt,
        address: u16,
        value: u8,
    ) {
        match address & 7 {
            0 => {
                let nmi_active = (value & 0x80) != 0;

                if !nmi_active {
                    *interrupt &= !INT_NMI;
                } else if self.nmi_occurred && !self.nmi_active {
                    *interrupt |= INT_NMI;
                }

                self.nmi_active = nmi_active;
                self.control.bg_chr_offset = if (value & 0x10) != 0 { 0x1000 } else { 0 };
                self.control.vram_increment = if (value & 0x04) != 0 { 32 } else { 1 };
                self.regs.t = (self.regs.t & 0x73ff) | ((value as u16 & 0x03) << 10);

                debug!("PPU NMI Active: {}", self.nmi_active);
                debug!("PPU BG CHR Offset: {}", self.control.bg_chr_offset);
                debug!("PPU VRAM Increment: {}", self.control.vram_increment);
                debug!("PPU TMP Address: {:04X}", self.regs.t);
            }
            1 => {
                self.mask.render_enabled = (value & 0x18) != 0;

                self.mask.bg_start = match value & 0x0a {
                    0x0a => 0,
                    0x08 => 8,
                    _ => i32::MAX,
                };

                debug!("PPU Render Enabled: {}", self.mask.render_enabled);
                debug!("PPU BG Start: {}", self.mask.bg_start);
            }
            5 => {
                if self.regs.w {
                    self.regs.t = (self.regs.t & 0x0c1f)
                        | ((value as u16 & 0xf8) << 2)
                        | ((value as u16 & 0x07) << 12);
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                } else {
                    self.regs.t = (self.regs.t & 0x7fe0) | ((value >> 3) as u16);
                    self.regs.x = value & 0x07;
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                    debug!("PPU Fine X: {}", self.regs.x);
                }

                self.regs.w = !self.regs.w;
            }
            6 => {
                if self.regs.w {
                    self.regs.t = (self.regs.t & 0xff00) | value as u16;
                    self.regs.v = self.regs.t;
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                    debug!("PPU VRAM Address: {:04X}", self.regs.v);
                } else {
                    self.regs.t = (self.regs.t & 0xff) | ((value as u16 & 0x3f) << 8);
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                }

                self.regs.w = !self.regs.w;
            }
            7 => {
                let address = self.regs.v & 0x3fff;

                debug!("VRAM Write: {:04X} = {:02X}", address, value);

                if address >= 0x3f00 {
                    self.palette.write(address, value);
                } else if address >= 0x2000 {
                    cartridge.write_name(address, value);
                } else {
                    cartridge.write_chr(address, value);
                }

                self.regs.v = (self.regs.v + self.control.vram_increment) & 0x7fff;
            }
            _ => warn!("PPU write {:04X} not yet implemented", address),
        }
    }

    pub fn step(&mut self, cartridge: &mut Cartridge, interrupt: &mut Interrupt) {
        if self.line < TOTAL_VISIBLE_LINES && self.mask.render_enabled {
            match self.dot {
                0..=255 => {
                    if self.line != PRE_RENDER_LINE {
                        self.draw_pixel();
                    }

                    self.load_bg_tiles(cartridge);

                    if self.dot == 255 {
                        self.increment_vertical();
                    }
                }
                256..=319 => {
                    if self.dot == 256 {
                        self.copy_horizontal();
                    }

                    if self.line == PRE_RENDER_LINE && self.dot >= 279 && self.dot <= 303 {
                        self.copy_vertical();
                    }
                }
                320..=335 => {
                    self.load_bg_tiles(cartridge);
                }
                336..=339 => {
                    //
                }
                340 => {
                    self.next_line(interrupt);
                }
                _ => unreachable!(),
            }
        } else if self.dot == 340 {
            self.next_line(interrupt);
        }

        self.dot += 1;
    }

    fn next_line(&mut self, interrupt: &mut Interrupt) {
        self.dot = -1;
        self.line += 1;

        if self.line == VBLANK_LINE {
            self.screen.reset();
            self.ready = true;

            self.nmi_occurred = true;
            debug!("PPU NMI Occurred: {}", self.nmi_occurred);

            if self.nmi_active {
                *interrupt |= INT_NMI;
            }
        } else if self.line == MAX_LINE_NUMBER {
            self.line = PRE_RENDER_LINE;
            self.nmi_occurred = false;
            *interrupt &= !INT_NMI;
            debug!("PPU NMI Occurred: {}", self.nmi_occurred);
        }
    }
}

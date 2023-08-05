pub use screen::{HEIGHT, WIDTH};

use super::cartridge::Cartridge;
use super::interrupt::{Interrupt, InterruptType};
use crate::Mapped;
use oam::Oam;
use palette::Palette;
use render::RenderState;
use screen::Screen;
use tracing::{debug, warn};

mod oam;
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
    nmi_active: bool,
    bg_chr_offset: u16,
    sprite_size: bool,
    sprite_chr_offset: u16,
    vram_increment: u16,
}

struct Mask {
    render_enabled: bool,
    bg_start: i32,
    sprite_start: i32,
}
struct Status {
    nmi_occurred: bool,
    sprite_zero_hit: bool,
}

pub struct Ppu {
    ready: bool,
    line: i32,
    dot: i32,
    read_buffer: u8,
    sprites_selected: usize,
    sprite_zero_selected: bool,
    interrupt: Interrupt,
    regs: Registers,
    control: Control,
    status: Status,
    mask: Mask,
    render: RenderState,
    palette: Palette,
    screen: Screen,
    oam: Oam,
}

impl Ppu {
    pub fn new(interrupt: Interrupt) -> Self {
        Self {
            ready: false,
            line: 0,
            dot: 0,
            read_buffer: 0,
            sprites_selected: 0,
            sprite_zero_selected: false,
            interrupt,
            regs: Registers {
                v: 0,
                t: 0,
                x: 0,
                w: false,
            },
            control: Control {
                nmi_active: false,
                bg_chr_offset: 0,
                sprite_size: false,
                sprite_chr_offset: 0,
                vram_increment: 1,
            },
            status: Status {
                nmi_occurred: false,
                sprite_zero_hit: false,
            },
            mask: Mask {
                render_enabled: false,
                bg_start: 0,
                sprite_start: 0,
            },
            render: RenderState::new(),
            palette: Palette::new(),
            screen: Screen::new(),
            oam: Oam::new(),
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

    pub fn read(&mut self, cartridge: &mut Cartridge<impl Mapped>, address: u16) -> u8 {
        match address & 7 {
            2 => {
                // TODO: Open bus
                let mut result: u8 = 0;

                if self.status.nmi_occurred {
                    result |= 0x80;
                    self.status.nmi_occurred = false;
                    debug!("NMI Occurred: {}", self.status.nmi_occurred);
                    self.interrupt.clear(InterruptType::Nmi);
                }

                if self.status.sprite_zero_hit {
                    result |= 0x40;
                }

                self.regs.w = false;

                result
            }
            4 => self.oam.read(),
            7 => {
                let address = self.regs.v & 0x3fff;

                let value = if address >= 0x3f00 {
                    self.palette.read(address)
                } else {
                    self.read_buffer
                };

                self.read_buffer = cartridge.read_vram(address);

                debug!(
                    "VRAM Read: {:04X} => {:02X} ({:02X})",
                    address, value, self.read_buffer
                );

                self.regs.v = (self.regs.v + self.control.vram_increment) & 0x7fff;
                cartridge.on_ppu_address_changed(self.regs.v);

                value
            }
            _ => {
                debug!("Unmapped PPU read {:04X}:", address);
                // TODO: PPU Open Bus
                0
            }
        }
    }

    pub fn write(&mut self, cartridge: &mut Cartridge<impl Mapped>, address: u16, value: u8) {
        match address & 7 {
            0 => {
                let nmi_active = (value & 0x80) != 0;

                if !nmi_active {
                    self.interrupt.clear(InterruptType::Nmi);
                } else if nmi_active && self.status.nmi_occurred && !self.control.nmi_active {
                    self.interrupt.raise(InterruptType::Nmi);
                }

                self.control.nmi_active = nmi_active;
                self.control.sprite_size = (value & 0x20) != 0;
                self.control.bg_chr_offset = if (value & 0x10) != 0 { 0x1000 } else { 0 };
                self.control.sprite_chr_offset = if (value & 0x08) != 0 { 0x1000 } else { 0 };
                self.control.vram_increment = if (value & 0x04) != 0 { 32 } else { 1 };
                self.regs.t = (self.regs.t & 0x73ff) | ((value as u16 & 0x03) << 10);

                debug!("PPU NMI Active: {}", self.control.nmi_active);
                debug!("PPU Sprite Size: {}", 8 << self.control.sprite_size as u32);
                debug!("PPU BG CHR Offset: {}", self.control.bg_chr_offset);
                debug!("PPU Sprite CHR Offset: {}", self.control.sprite_chr_offset);
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

                self.mask.sprite_start = match value & 0x14 {
                    0x14 => 0,
                    0x10 => 8,
                    _ => i32::MAX,
                };

                debug!("PPU Render Enabled: {}", self.mask.render_enabled);
                debug!("PPU BG Start: {}", self.mask.bg_start);
                debug!("PPU Sprite Start: {}", self.mask.sprite_start);
            }
            3 => self.oam.set_address(value),
            4 => self.oam.write(value),
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
                    cartridge.on_ppu_address_changed(self.regs.v);
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
                } else {
                    cartridge.write_vram(address, value);
                }

                self.regs.v = (self.regs.v + self.control.vram_increment) & 0x7fff;
                cartridge.on_ppu_address_changed(self.regs.v);
            }
            _ => warn!("PPU write {:04X} not yet implemented", address),
        }
    }

    pub fn write_oam(&mut self, value: u8) {
        self.oam.write(value);
    }

    pub fn step(&mut self, cartridge: &mut Cartridge<impl Mapped>) {
        if self.line < TOTAL_VISIBLE_LINES {
            if self.mask.render_enabled {
                self.render(cartridge);
            } else if self.line != PRE_RENDER_LINE && self.dot < 256 {
                // TODO: The backdrop colour can apparently be set using palette address?
                self.screen.draw(self.palette.color(0));
            }
        }

        if self.dot == 340 {
            self.next_line();
        } else {
            self.dot += 1;
        }
    }

    fn next_line(&mut self) {
        self.dot = 0;
        self.line += 1;

        if self.line == VBLANK_LINE {
            self.screen.reset();
            self.ready = true;

            self.status.nmi_occurred = true;
            debug!("PPU NMI Occurred: {}", self.status.nmi_occurred);

            if self.control.nmi_active {
                self.interrupt.raise(InterruptType::Nmi);
            }
        } else if self.line == MAX_LINE_NUMBER {
            self.line = PRE_RENDER_LINE;

            self.status.nmi_occurred = false;
            debug!("PPU NMI Occurred: {}", self.status.nmi_occurred);

            self.interrupt.clear(InterruptType::Nmi);

            self.status.sprite_zero_hit = false;
            debug!("Sprite Zero Hit: {}", self.status.sprite_zero_hit);
        }
    }
}

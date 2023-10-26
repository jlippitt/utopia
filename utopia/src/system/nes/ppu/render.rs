use super::super::cartridge::Cartridge;
use crate::Mapped;
use tracing::trace;

const ATTR_SHIFT: [u32; 4] = [0, 0x5555, 0xaaaa, 0xffff];

#[derive(Clone, Copy, Debug, Default)]
struct Sprite {
    x: i32,
    chr_low: u8,
    chr_high: u8,
    attr: u8,
}

pub struct RenderState {
    address: u16,
    name: u16,
    attr_latch: u8,
    chr_latch: u8,
    chr_low: u32,
    chr_high: u32,
    attr_shift: u32,
    sprite_y: u16,
    sprite_name: u16,
    sprites: [Sprite; 8],
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            address: 0,
            name: 0,
            attr_latch: 0,
            chr_latch: 0,
            chr_low: 0,
            chr_high: 0,
            attr_shift: 0,
            sprite_y: 0,
            sprite_name: 0,
            sprites: [Default::default(); 8],
        }
    }

    pub fn set_address(&mut self, cartridge: &mut Cartridge<impl Mapped>, address: u16) {
        self.address = address;
        cartridge.on_ppu_address_changed(self.address);
    }
}

impl super::Ppu {
    pub(super) fn render(&mut self, cartridge: &mut Cartridge<impl Mapped>) {
        match self.dot {
            0..=255 => {
                if self.line != super::PRE_RENDER_LINE {
                    self.draw_pixel();

                    // TODO: Precise timings for sprite operations
                    if self.dot == 255 {
                        (self.sprites_selected, self.sprite_zero_selected) =
                            self.oam.select_sprites(self.line, self.control.sprite_size);
                    }
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

                if self.line == super::PRE_RENDER_LINE && self.dot >= 279 && self.dot <= 303 {
                    self.copy_vertical();
                }

                self.load_sprite_tiles(cartridge);
            }
            320..=335 => {
                self.load_bg_tiles(cartridge);
            }
            336..=339 => {
                self.load_extra(cartridge);
            }
            _ => (),
        }
    }

    fn draw_pixel(&mut self) {
        // 1. Backdrop Color
        let mut color = self.palette.color(0);
        let mut bg_present = false;
        let mut sprite_present = false;

        // 2. Background Layer
        if self.dot >= self.mask.bg_start {
            let shift = 15 - self.regs.x;
            let low = (self.render.chr_low >> shift) & 0x01;
            let high = (self.render.chr_high >> shift) & 0x02;
            let value = high | low;

            if value > 0 {
                let attr = (self.render.attr_shift >> (shift << 1)) & 0x03;
                let index = (attr << 2) | high | low;
                color = self.palette.color(index as usize);
                bg_present = true;
            }
        }

        // 3. Sprite Layer
        let sprites = &mut self.render.sprites[0..self.sprites_selected];

        for (index, sprite) in sprites.iter_mut().enumerate() {
            sprite.x -= 1;

            if sprite.x < 0 || sprite.x >= 8 || self.dot < self.mask.sprite_start {
                continue;
            }

            let shift = if sprite.attr & 0x40 != 0 {
                sprite.x ^ 7
            } else {
                sprite.x
            };

            let low = (sprite.chr_low >> shift) & 0x01;
            let high = (sprite.chr_high >> shift) & 0x01;
            let value = (high << 1) | low;

            if value == 0 {
                continue;
            }

            if index == 0 && self.sprite_zero_selected && bg_present && self.dot != 255 {
                self.status.sprite_zero_hit = true;
                trace!("Sprite Zero Hit: {}", self.status.sprite_zero_hit);
            }

            if sprite_present {
                continue;
            }

            // Sprite priority quirk: Lower priority sprites cannot overlap non-transparent pixels
            // of higher priority sprites, even if they are hidden by the background layer.
            if (sprite.attr & 0x20) == 0 || !bg_present {
                let index = 0x10 | ((sprite.attr & 0x03) << 2) | value;
                color = self.palette.color(index as usize);
            }

            sprite_present = true;
        }

        self.screen.draw(color);
    }

    fn load_bg_tiles(&mut self, cartridge: &mut Cartridge<impl Mapped>) {
        self.render.chr_low <<= 1;
        self.render.chr_high <<= 1;
        self.render.attr_shift <<= 2;

        match self.dot & 7 {
            0 => self.render.set_address(cartridge, self.tile_address()),
            1 => {
                let value = cartridge.read_vram(self.render.address);
                self.render.name = (value as u16) << 4;
            }
            2 => self.render.set_address(cartridge, self.attr_address()),
            3 => {
                let value = cartridge.read_vram(self.render.address);
                let shift = ((self.regs.v & 0x40) >> 4) | (self.regs.v & 0x02);
                self.render.attr_latch = (value >> shift) & 0x03;
            }
            4 => self.render.set_address(cartridge, self.bg_chr_address()),
            5 => {
                self.render.chr_latch = cartridge.read_vram(self.render.address);
            }
            6 => self
                .render
                .set_address(cartridge, self.bg_chr_address() | 0x08),
            7 => {
                let value = cartridge.read_vram(self.render.address);

                // TODO: This is actually meant to be one cycle later, but that makes timing more complex
                self.render.chr_low =
                    (self.render.chr_low & 0xff00) | (self.render.chr_latch as u32);

                self.render.chr_high = (self.render.chr_high & 0x0001_fe00) | ((value as u32) << 1);

                self.render.attr_shift = (self.render.attr_shift & 0xffff_0000)
                    | ATTR_SHIFT[self.render.attr_latch as usize];

                self.increment_horizontal();
            }
            _ => unreachable!(),
        }
    }

    fn load_sprite_tiles(&mut self, cartridge: &mut Cartridge<impl Mapped>) {
        let index = (self.dot as usize >> 3) & 7;
        let address = index << 2;

        match self.dot & 7 {
            0 => {
                self.render.set_address(cartridge, self.tile_address());
                self.render.sprite_y = self.oam.read_secondary(address) as u16;
            }
            1 => {
                cartridge.read_vram(self.render.address);
                self.render.sprite_name = (self.oam.read_secondary(address + 1) as u16) << 4;
            }
            2 => {
                self.render.set_address(cartridge, self.attr_address());
                self.render.sprites[index].attr = self.oam.read_secondary(address + 2);
            }
            3 => {
                cartridge.read_vram(self.render.address);
                self.render.sprites[index].x = self.oam.read_secondary(address + 3) as i32 + 8;
            }
            4 => {
                self.render.sprites[index].x = self.oam.read_secondary(address + 3) as i32 + 8;
                self.render
                    .set_address(cartridge, self.sprite_chr_address(index));
            }
            5 => {
                self.render.sprites[index].x = self.oam.read_secondary(address + 3) as i32 + 8;
                self.render.sprites[index].chr_low = cartridge.read_vram(self.render.address);
            }
            6 => {
                self.render.sprites[index].x = self.oam.read_secondary(address + 3) as i32 + 8;
                self.render
                    .set_address(cartridge, self.sprite_chr_address(index) | 0x08);
            }
            7 => {
                self.render.sprites[index].x = self.oam.read_secondary(address + 3) as i32 + 8;
                self.render.sprites[index].chr_high = cartridge.read_vram(self.render.address);
            }
            _ => unreachable!(),
        }
    }

    fn load_extra(&mut self, cartridge: &mut Cartridge<impl Mapped>) {
        match self.dot & 1 {
            0 => self.render.set_address(cartridge, self.tile_address()),
            1 => {
                let value = cartridge.read_vram(self.render.address);
                self.render.name = (value as u16) << 4;
            }
            _ => unreachable!(),
        }
    }

    fn copy_horizontal(&mut self) {
        self.regs.v = (self.regs.v & 0x7be0) | (self.regs.t & 0x041f);
        trace!("PPU VRAM Address (Copy Horizontal): {:04X}", self.regs.v);
    }

    fn copy_vertical(&mut self) {
        self.regs.v = (self.regs.v & 0x041f) | (self.regs.t & 0x7be0);
        trace!("PPU VRAM Address (Copy Vertical): {:04X}", self.regs.v);
    }

    fn increment_horizontal(&mut self) {
        if self.regs.v & 0x1f == 0x1f {
            self.regs.v &= !0x1f;
            self.regs.v ^= 0x0400;
        } else {
            self.regs.v += 1;
        }
    }

    fn increment_vertical(&mut self) {
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

        trace!("PPU VRAM Address (Increment Vertical): {:04X}", self.regs.v);
    }

    fn tile_address(&self) -> u16 {
        0x2000 | (self.regs.v & 0x0fff)
    }

    fn attr_address(&self) -> u16 {
        0x23c0 | (self.regs.v & 0x0c00) | ((self.regs.v >> 4) & 0x38) | ((self.regs.v >> 2) & 0x07)
    }

    fn bg_chr_address(&self) -> u16 {
        self.control.bg_chr_offset | self.render.name | (self.regs.v >> 12)
    }

    fn sprite_chr_address(&self, index: usize) -> u16 {
        let mut row = (self.line as u16).wrapping_sub(self.render.sprite_y) & 15;
        let flip = (self.render.sprites[index].attr & 0x80) != 0;

        if self.control.sprite_size {
            if flip {
                row ^= 15;
            }

            ((self.render.sprite_name & 0x10) << 8)
                | (self.render.sprite_name & 0x0fe0)
                | ((row & 0x08) << 1)
                | (row & 0x07)
        } else {
            if flip {
                row ^= 7;
            }

            self.control.sprite_chr_offset | self.render.sprite_name | row
        }
    }
}

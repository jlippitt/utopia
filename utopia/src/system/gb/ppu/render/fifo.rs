use super::super::oam::Sprite;
use bitfield_struct::bitfield;

#[bitfield(u8)]
pub struct BgAttrByte {
    #[bits(3)]
    pub palette: u8,
    pub bank: bool,
    __: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub above_obj: bool,
}

#[derive(Copy, Clone, Default)]
pub struct BgPixel {
    pub color: u8,
    pub below_bg: bool,
    pub palette: bool,
}

#[derive(Default)]
pub struct BackgroundFifo {
    chr: (u8, u8),
    attr: BgAttrByte,
    remaining: u8,
}

impl BackgroundFifo {
    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    pub fn attr(&self) -> BgAttrByte {
        self.attr
    }

    pub fn clear(&mut self) {
        self.remaining = 0;
    }

    pub fn try_push(&mut self, chr: (u8, u8), attr: BgAttrByte) -> bool {
        if self.remaining == 0 {
            self.chr = chr;
            self.attr = attr;
            self.remaining = 8;

            if attr.flip_x() {
                self.chr.0 = self.chr.0.reverse_bits();
                self.chr.1 = self.chr.1.reverse_bits();
            }

            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.remaining != 0 {
            let color = ((self.chr.0 >> 7) & 1) + ((self.chr.1 >> 6) & 2);
            self.chr.0 <<= 1;
            self.chr.1 <<= 1;
            self.remaining -= 1;
            Some(color)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct SpritePixel {
    pub priority: u8,
    pub color: u8,
    pub below_bg: bool,
    pub palette: u8,
}

#[derive(Default)]
pub struct SpriteFifo {
    pixels: [SpritePixel; 8],
    read_index: usize,
}

impl SpriteFifo {
    pub fn push(&mut self, sprite: &Sprite, chr: (u8, u8), is_cgb: bool) {
        let colors: [u8; 8] = [
            ((chr.0 >> 7) & 1) | ((chr.1 >> 6) & 2),
            ((chr.0 >> 6) & 1) | ((chr.1 >> 5) & 2),
            ((chr.0 >> 5) & 1) | ((chr.1 >> 4) & 2),
            ((chr.0 >> 4) & 1) | ((chr.1 >> 3) & 2),
            ((chr.0 >> 3) & 1) | ((chr.1 >> 2) & 2),
            ((chr.0 >> 2) & 1) | ((chr.1 >> 1) & 2),
            ((chr.0 >> 1) & 1) | ((chr.1 >> 0) & 2),
            ((chr.0 >> 0) & 1) | ((chr.1 << 1) & 2),
        ];

        let total_pixels = (sprite.x as usize).min(8);

        for write_index in 0..total_pixels {
            let pixel = &mut self.pixels[(self.read_index + write_index) & 7];

            let color = if sprite.attr.flip_x() {
                colors[total_pixels - write_index - 1]
            } else {
                colors[8 - total_pixels + write_index]
            };

            if pixel.color != 0 && (!is_cgb || color == 0 || pixel.priority <= sprite.id) {
                continue;
            }

            *pixel = SpritePixel {
                priority: sprite.id,
                color,
                below_bg: sprite.attr.below_bg(),
                palette: if is_cgb {
                    sprite.attr.cgb_palette()
                } else {
                    sprite.attr.dmg_palette() as u8
                },
            };
        }
    }

    pub fn pop(&mut self) -> SpritePixel {
        let pixel = self.pixels[self.read_index];
        self.pixels[self.read_index] = Default::default();
        self.read_index = (self.read_index + 1) & 7;
        pixel
    }
}

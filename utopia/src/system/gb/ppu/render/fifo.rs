use super::super::oam::Sprite;
use std::cmp;

pub struct BackgroundFifo {
    chr: (u8, u8),
    remaining: u8,
}

impl BackgroundFifo {
    pub fn new() -> Self {
        Self {
            chr: (0, 0),
            remaining: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    pub fn try_push(&mut self, chr: (u8, u8)) -> bool {
        if self.remaining == 0 {
            self.chr = chr;
            self.remaining = 8;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.remaining != 0 {
            let pixel = ((self.chr.0 >> 7) & 1) + ((self.chr.1 >> 6) & 2);
            self.chr.0 <<= 1;
            self.chr.1 <<= 1;
            self.remaining -= 1;
            Some(pixel)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct SpritePixel {
    pub color: u8,
    pub below_bg: bool,
    pub palette: bool,
}

pub struct SpriteFifo {
    pixels: [SpritePixel; 8],
    read_index: usize,
}

impl SpriteFifo {
    pub fn new() -> Self {
        Self {
            pixels: [Default::default(); 8],
            read_index: 0,
        }
    }

    pub fn push(&mut self, sprite: &Sprite, chr: (u8, u8)) {
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

        let total_pixels = cmp::min(8, sprite.x as usize);

        for write_index in 0..total_pixels {
            let pixel = &mut self.pixels[(self.read_index + write_index) & 7];

            if pixel.color != 0 {
                continue;
            }

            let color = if sprite.flip_x {
                colors[total_pixels - write_index - 1]
            } else {
                colors[8 - total_pixels + write_index]
            };

            *pixel = SpritePixel {
                color,
                below_bg: sprite.below_bg,
                palette: sprite.palette,
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

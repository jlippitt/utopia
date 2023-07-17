use super::super::oam::Sprite;

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

    pub fn try_push(&mut self, chr: (u8, u8)) -> bool {
        if self.remaining == 0 {
            self.chr = chr;
            self.remaining = 8;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<(u8, u8)> {
        if self.remaining != 0 {
            let pixel = ((self.chr.0 >> 7) & 1, (self.chr.1 >> 7) & 1);
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
}

pub struct SpriteFifo {
    pixels: [SpritePixel; 8],
    index: usize,
}

impl SpriteFifo {
    pub fn new() -> Self {
        Self {
            pixels: [Default::default(); 8],
            index: 0,
        }
    }

    pub fn push(&mut self, sprite: &Sprite, chr: (u8, u8)) {
        //
    }
}

use super::palette::Color;
use crate::util::Rgb;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const DMG_COLORS: [Rgb; 4] = [
    Rgb(255, 255, 255),
    Rgb(169, 169, 169),
    Rgb(84, 84, 84),
    Rgb(0, 0, 0),
];

pub struct Screen {
    pixels: [u8; WIDTH * HEIGHT * 4],
    index: usize,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: [0; WIDTH * HEIGHT * 4],
            index: 0,
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn draw_pixel_dmg(&mut self, color: u8) {
        let rgb = DMG_COLORS[color as usize];

        self.pixels[self.index] = rgb.0;
        self.pixels[self.index + 1] = rgb.1;
        self.pixels[self.index + 2] = rgb.2;
        self.pixels[self.index + 3] = 0xff;
        self.index += 4;
    }

    pub fn draw_pixel_cgb(&mut self, color: Color) {
        self.pixels[self.index] = color.red() << 3;
        self.pixels[self.index + 1] = color.green() << 3;
        self.pixels[self.index + 2] = color.blue() << 3;
        self.pixels[self.index + 3] = 0xff;
        self.index += 4;
    }
}

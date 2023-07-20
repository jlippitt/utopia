use super::buffer::PixelBuffer;

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 448;

const PITCH: usize = WIDTH * 4;

pub struct Screen {
    pixels: Vec<u8>,
    index: usize,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; PITCH * HEIGHT],
            index: 0,
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn draw_lo_res(&mut self, pixels_main: &PixelBuffer) {
        for pixel in pixels_main {
            self.output_pixel(pixel.color);
            self.output_pixel(pixel.color);
        }

        self.pixels
            .copy_within((self.index - PITCH)..self.index, self.index);

        self.index += PITCH;
    }

    fn output_pixel(&mut self, color: u16) {
        let output = &mut self.pixels[self.index..(self.index + 4)];
        output[0] = ((color & 0x001f) << 3) as u8;
        output[1] = ((color & 0x03e0) >> 2) as u8;
        output[2] = ((color & 0x7c00) >> 7) as u8;
        output[3] = 0xff;
        self.index += 4;
    }
}

use super::super::vce::Color;
use tracing::debug;

pub struct Screen {
    width: u16,
    height: u16,
    index: usize,
    pixels: Vec<u8>,
}

impl Screen {
    pub const DEFAULT_WIDTH: u16 = 256;
    pub const DEFAULT_HEIGHT: u16 = 224;

    pub fn new() -> Self {
        Self {
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            index: 0,
            pixels: vec![0u8; Self::DEFAULT_WIDTH as usize * Self::DEFAULT_HEIGHT as usize * 4],
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn set_width(&mut self, msb: bool, value: u8) {
        if !msb {
            self.width = ((value as u16 & 0x3f) + 1) << 3;
            debug!("VDC Display Width: {}", self.width);
            self.resize_pixel_buffer();
        }
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn set_height(&mut self, msb: bool, value: u8) {
        let last_display_line = if msb {
            ((self.height - 1) & 0xff) | ((value as u16 & 0x01) << 8)
        } else {
            ((self.height - 1) & 0xff00) | (value as u16)
        };

        self.height = last_display_line + 1;

        debug!("VDC Display Height: {}", self.height);

        self.resize_pixel_buffer();
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn draw_line(&mut self, line_buffer: &[Color]) {
        for color in line_buffer {
            let output = &mut self.pixels[self.index..(self.index + 4)];
            output[0] = color.red() << 5;
            output[1] = color.green() << 5;
            output[2] = color.blue() << 5;
            output[3] = 0xff;
            self.index += 4;
        }
    }

    fn resize_pixel_buffer(&mut self) {
        let new_size = self.width as usize * self.height as usize * 4;

        if self.pixels.len() != new_size {
            self.pixels.resize(new_size, 0);
        }
    }
}

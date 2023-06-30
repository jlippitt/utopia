pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

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

    pub fn draw_pixel(&mut self, _color: u8) {
        self.pixels[self.index] = 0xff;
        self.pixels[self.index + 1] = 0xff;
        self.pixels[self.index + 2] = 0xff;
        self.pixels[self.index + 3] = 0xff;
        self.index += 4;
    }
}

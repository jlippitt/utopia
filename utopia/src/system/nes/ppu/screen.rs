pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct Screen {
    pixels: Vec<u8>,
    index: usize,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: vec![0u8; WIDTH * HEIGHT * 4],
            index: 0,
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn draw(&mut self) {
        let slice = &mut self.pixels[self.index..(self.index + 4)];

        slice[0] = 0x00;
        slice[1] = 0x00;
        slice[2] = 0xff;
        slice[3] = 0xff;

        self.index += 4;
    }
}
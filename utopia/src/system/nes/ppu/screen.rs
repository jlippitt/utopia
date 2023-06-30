use crate::util::Rgb;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

const RGB_VALUES: [Rgb; 64] = [
    // Dark
    Rgb(84, 84, 84),
    Rgb(0, 30, 116),
    Rgb(8, 16, 144),
    Rgb(48, 0, 136),
    Rgb(68, 0, 100),
    Rgb(92, 0, 48),
    Rgb(84, 4, 0),
    Rgb(60, 24, 0),
    Rgb(32, 42, 0),
    Rgb(8, 58, 0),
    Rgb(0, 64, 0),
    Rgb(0, 60, 0),
    Rgb(0, 50, 60),
    Rgb(0, 0, 0),
    Rgb(0, 0, 0),
    Rgb(0, 0, 0),
    // Medium
    Rgb(152, 150, 152),
    Rgb(8, 76, 196),
    Rgb(48, 50, 236),
    Rgb(92, 30, 228),
    Rgb(136, 20, 176),
    Rgb(160, 20, 100),
    Rgb(152, 34, 32),
    Rgb(120, 60, 0),
    Rgb(84, 90, 0),
    Rgb(40, 114, 0),
    Rgb(8, 124, 0),
    Rgb(0, 118, 40),
    Rgb(0, 102, 120),
    Rgb(0, 0, 0),
    Rgb(0, 0, 0),
    Rgb(0, 0, 0),
    // Light
    Rgb(236, 238, 236),
    Rgb(76, 154, 236),
    Rgb(120, 124, 236),
    Rgb(176, 98, 236),
    Rgb(228, 84, 236),
    Rgb(236, 88, 180),
    Rgb(236, 106, 100),
    Rgb(212, 136, 32),
    Rgb(160, 170, 0),
    Rgb(116, 196, 0),
    Rgb(76, 208, 32),
    Rgb(56, 204, 108),
    Rgb(56, 180, 204),
    Rgb(60, 60, 60),
    Rgb(0, 0, 0),
    Rgb(0, 0, 0),
    // Pale
    Rgb(236, 238, 236),
    Rgb(168, 204, 236),
    Rgb(188, 188, 236),
    Rgb(212, 178, 236),
    Rgb(236, 174, 236),
    Rgb(236, 174, 212),
    Rgb(236, 180, 176),
    Rgb(228, 196, 144),
    Rgb(204, 210, 120),
    Rgb(180, 222, 120),
    Rgb(168, 226, 144),
    Rgb(152, 226, 180),
    Rgb(160, 214, 228),
    Rgb(160, 162, 160),
    Rgb(0, 0, 0),
    Rgb(0, 0, 0),
];

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

    pub fn draw(&mut self, color: u8) {
        let slice = &mut self.pixels[self.index..(self.index + 4)];

        let rgb = RGB_VALUES[color as usize];

        slice[0] = rgb.0;
        slice[1] = rgb.1;
        slice[2] = rgb.2;
        slice[3] = 0xff;

        self.index += 4;
    }
}

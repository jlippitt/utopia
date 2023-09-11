use crate::util::Rgb;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const DMG_COLORS: [Rgb; 4] = [
    Rgb(255, 255, 255),
    Rgb(169, 169, 169),
    Rgb(84, 84, 84),
    Rgb(0, 0, 0),
];

const CGB_TABLE_SIZE: usize = 32768;

const CGB_COLORS: [Rgb; CGB_TABLE_SIZE] = create_cgb_table();

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

    pub fn draw_pixel_cgb(&mut self, color: u16) {
        let rgb = CGB_COLORS[color as usize];
        self.pixels[self.index] = rgb.0;
        self.pixels[self.index + 1] = rgb.1;
        self.pixels[self.index + 2] = rgb.2;
        self.pixels[self.index + 3] = 0xff;
        self.index += 4;
    }
}

const fn create_cgb_table() -> [Rgb; CGB_TABLE_SIZE] {
    let mut table = [Rgb(0, 0, 0); CGB_TABLE_SIZE];
    let mut index = 0;

    while index < CGB_TABLE_SIZE {
        let red = index & 0x1f;
        let green = (index >> 5) & 0x1f;
        let blue = (index >> 10) & 0x1f;

        table[index] = Rgb(
            ((red * 13 + green * 2 + blue) >> 1) as u8,
            ((green * 3 + blue) << 1) as u8,
            ((red * 3 + green * 2 + blue * 11) >> 1) as u8,
        );

        index += 1;
    }

    table
}

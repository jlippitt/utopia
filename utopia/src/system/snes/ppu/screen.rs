use super::buffer::PixelBuffer;
use tracing::trace;

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 448;

const PITCH: usize = WIDTH * 4;

const GAMMA_TABLE: [u8; 32] = [
    0x00, 0x01, 0x03, 0x06, 0x0a, 0x0f, 0x15, 0x1c, 0x24, 0x2d, 0x37, 0x42, 0x4e, 0x5b, 0x69, 0x78,
    0x88, 0x90, 0x98, 0xa0, 0xa8, 0xb0, 0xb8, 0xc0, 0xc8, 0xd0, 0xd8, 0xe0, 0xe8, 0xf0, 0xf8, 0xff,
];

const BRIGHTNESS_LEVELS: usize = 16;
const INTENSITY_LEVELS: usize = 32;

type IntensityTable = [u8; INTENSITY_LEVELS];
type BrightnessTable = [IntensityTable; BRIGHTNESS_LEVELS];

const fn generate_brightness_table() -> BrightnessTable {
    let mut table = [[0; INTENSITY_LEVELS]; BRIGHTNESS_LEVELS];
    let mut brightness = 0;

    while brightness < BRIGHTNESS_LEVELS {
        let mut intensity = 0;

        while intensity < INTENSITY_LEVELS {
            table[brightness][intensity] = GAMMA_TABLE[(intensity * (brightness + 1)) >> 4];
            intensity += 1;
        }

        brightness += 1;
    }

    table
}

const BRIGHTNESS_TABLE: BrightnessTable = generate_brightness_table();

pub struct Screen {
    output: Vec<u8>,
    index: usize,
    intensity: &'static IntensityTable,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            output: vec![0; PITCH * HEIGHT],
            index: 0,
            intensity: &BRIGHTNESS_TABLE[0],
        }
    }

    pub fn output(&self) -> &[u8] {
        &self.output
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn set_brightness(&mut self, value: u8) {
        self.intensity = &BRIGHTNESS_TABLE[value as usize];
        trace!("Brightness: {}", value);
    }

    pub fn draw_lo_res(&mut self, main_screen: &PixelBuffer) {
        for pixel in main_screen {
            let intensity = &self.intensity;
            let output = &mut self.output[self.index..(self.index + 8)];

            let (red, green, blue) = rgb(intensity, pixel.color);
            output[0] = red;
            output[1] = green;
            output[2] = blue;
            output[3] = 0xff;
            output[4] = red;
            output[5] = green;
            output[6] = blue;
            output[7] = 0xff;

            self.index += 8;
        }
    }

    pub fn draw_hi_res(&mut self, pixels: &[PixelBuffer; 2]) {
        let [main_screen, sub_screen] = &pixels;

        for (pixel_main, pixel_sub) in main_screen.iter().zip(sub_screen) {
            let intensity = &self.intensity;
            let output = &mut self.output[self.index..(self.index + 8)];

            {
                let (red, green, blue) = rgb(intensity, pixel_main.color);
                output[0] = red;
                output[1] = green;
                output[2] = blue;
                output[3] = 0xff;
            }

            {
                let (red, green, blue) = rgb(intensity, pixel_sub.color);
                output[4] = red;
                output[5] = green;
                output[6] = blue;
                output[7] = 0xff;
            }

            self.index += 8;
        }
    }

    pub fn force_blank(&mut self) {
        let end = self.index + PITCH;

        // This creates a repeating [0, 0, 0, 0xff] sequence
        let iter = (0..PITCH).map(|index| ((index == 3) as u8).wrapping_neg());

        self.output.splice(self.index..end, iter);

        self.index = end;
    }

    pub fn skip_line(&mut self) {
        self.index += PITCH;
    }

    pub fn duplicate_line(&mut self) {
        self.output
            .copy_within((self.index - PITCH)..self.index, self.index);

        self.index += PITCH;
    }
}

fn rgb(intensity: &IntensityTable, color: u16) -> (u8, u8, u8) {
    let color = color as usize;
    let red = intensity[color & 31];
    let green = intensity[(color >> 5) & 31];
    let blue = intensity[(color >> 10) & 31];
    (red, green, blue)
}

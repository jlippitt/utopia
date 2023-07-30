use super::buffer::Pixel;
use super::window::MASK_NONE;
use tracing::debug;

const PRIORITY: u8 = 2;

pub struct Mode7Settings {
    matrix_a: i32,
    matrix_b: i32,
    matrix_c: i32,
    matrix_d: i32,
    center_x: i32,
    center_y: i32,
    scroll_x: i32,
    scroll_y: i32,
    flip_x: bool,
    flip_y: u16,
    transparency_fill: bool,
    tile_zero_fill: bool,
    write_buffer: u8,
}

impl Mode7Settings {
    pub fn new() -> Self {
        Self {
            matrix_a: 0,
            matrix_b: 0,
            matrix_c: 0,
            matrix_d: 0,
            center_x: 0,
            center_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            flip_x: false,
            flip_y: 0,
            transparency_fill: false,
            tile_zero_fill: false,
            write_buffer: 0,
        }
    }

    pub fn multiply(&self) -> i32 {
        let rhs = self.matrix_b >> 8;
        let result = self.matrix_a * rhs;

        debug!(
            "Multiplication (Signed): {} * {} = {}",
            self.matrix_a, rhs, result
        );

        result
    }

    pub fn set_control(&mut self, value: u8) {
        self.flip_x = (value & 0x01) != 0;
        self.flip_y = if (value & 0x02) != 0 { 255 } else { 0 };
        self.transparency_fill = (value & 0xc0) == 0x80;
        self.tile_zero_fill = (value & 0xc0) == 0xc0;
        debug!("Mode 7 Flip X: {}", self.flip_x);
        debug!("Mode 7 Flip Y: {}", self.flip_y);
        debug!("Mode 7 Transparency Fill: {}", self.transparency_fill);
        debug!("Mode 7 Tile Zero Fill: {}", self.tile_zero_fill);
    }

    pub fn set_matrix_a(&mut self, value: u8) {
        self.matrix_a = (self.word_value(value) as i16) as i32;
        debug!("Mode 7 Matrix A: {}", self.matrix_a);
    }

    pub fn set_matrix_b(&mut self, value: u8) {
        self.matrix_b = (self.word_value(value) as i16) as i32;
        debug!("Mode 7 Matrix B: {}", self.matrix_b);
    }

    pub fn set_matrix_c(&mut self, value: u8) {
        self.matrix_c = (self.word_value(value) as i16) as i32;
        debug!("Mode 7 Matrix C: {}", self.matrix_c);
    }

    pub fn set_matrix_d(&mut self, value: u8) {
        self.matrix_d = (self.word_value(value) as i16) as i32;
        debug!("Mode 7 Matrix D: {}", self.matrix_d);
    }

    pub fn set_center_x(&mut self, value: u8) {
        self.center_x = sign_extend_13(self.word_value(value));
        debug!("Mode 7 Center X: {}", self.center_x);
    }

    pub fn set_center_y(&mut self, value: u8) {
        self.center_y = sign_extend_13(self.word_value(value));
        debug!("Mode 7 Center Y: {}", self.center_y);
    }

    pub fn set_scroll_x(&mut self, value: u8) {
        self.scroll_x = sign_extend_13(self.word_value(value));
        debug!("Mode 7 Scroll X: {}", self.scroll_x);
    }

    pub fn set_scroll_y(&mut self, value: u8) {
        self.scroll_y = sign_extend_13(self.word_value(value));
        debug!("Mode 7 Scroll Y: {}", self.scroll_y);
    }

    fn word_value(&mut self, value: u8) -> u16 {
        let word_value = u16::from_le_bytes([self.write_buffer, value]);
        self.write_buffer = value;
        word_value
    }
}

impl super::Ppu {
    pub(super) fn draw_mode7(&mut self, bg_index: usize, line: u16) {
        let enabled = self.enabled[bg_index];

        for pixel_buffer_index in [0, 1] {
            if enabled.has(pixel_buffer_index) {
                self.draw_mode7_pixels(bg_index, pixel_buffer_index, line);
            }
        }
    }

    fn draw_mode7_pixels(&mut self, bg_index: usize, pixel_buffer_index: usize, line: u16) {
        let mask = if self.window_enabled[bg_index].has(pixel_buffer_index) {
            self.window_mask[bg_index].mask(&self.window)
        } else {
            &MASK_NONE
        };

        let pixels = &mut self.pixels[pixel_buffer_index];

        let offset_x = clip_13(self.mode7.scroll_x - self.mode7.center_x);
        let offset_y = clip_13(self.mode7.scroll_y - self.mode7.center_y);
        let flipped_y = (line ^ self.mode7.flip_y) as i32;

        let mut pos_x = ((self.mode7.matrix_a * offset_x) & !63)
            + ((self.mode7.matrix_b * offset_y) & !63)
            + ((self.mode7.matrix_b * flipped_y) & !63)
            + (self.mode7.center_x << 8);

        let mut pos_y = ((self.mode7.matrix_c * offset_x) & !63)
            + ((self.mode7.matrix_d * offset_y) & !63)
            + ((self.mode7.matrix_d * flipped_y) & !63)
            + (self.mode7.center_y << 8);

        let (increment_x, increment_y) = if self.mode7.flip_x {
            pos_x += self.mode7.matrix_a * 255;
            pos_y += self.mode7.matrix_c * 255;
            (-self.mode7.matrix_a, -self.mode7.matrix_c)
        } else {
            (self.mode7.matrix_a, self.mode7.matrix_c)
        };

        for (index, pixel) in pixels.iter_mut().enumerate() {
            let pixel_x = (pos_x >> 8) as usize;
            let pixel_y = (pos_y >> 8) as usize;
            let out_of_bounds = ((pixel_x | pixel_y) & !1023) != 0;

            if !out_of_bounds || !self.mode7.transparency_fill {
                let fine_x = pixel_x & 7;
                let fine_y = pixel_y & 7;

                let name = if !out_of_bounds || !self.mode7.tile_zero_fill {
                    let coarse_x = (pixel_x >> 3) & 0x7f;
                    let coarse_y = (pixel_y >> 3) & 0x7f;
                    self.vram.data((coarse_y << 7) | coarse_x) as usize & 0xff
                } else {
                    0
                };

                let color = self.vram.data((name << 6) + (fine_y << 3) + fine_x) >> 8;

                if color != 0 && !mask[index] && PRIORITY > pixel.priority {
                    *pixel = Pixel {
                        color: self.cgram.color(color as usize),
                        priority: PRIORITY,
                        layer: 1 << bg_index,
                    }
                }
            }

            pos_x += increment_x;
            pos_y += increment_y;
        }
    }
}

fn sign_extend_13(value: u16) -> i32 {
    ((value & 0x1fff).wrapping_sub((value & 0x1000) << 1) as i16) as i32
}

fn clip_13(value: i32) -> i32 {
    if (value & 0x2000) != 0 {
        value | !1023
    } else {
        value & 1023
    }
}

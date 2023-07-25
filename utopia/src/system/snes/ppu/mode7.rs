use tracing::debug;

pub struct Mode7Settings {
    matrix_a: i32,
    matrix_b: i32,
    matrix_c: i32,
    matrix_d: i32,
    center_x: i32,
    center_y: i32,
    scroll_x: i32,
    scroll_y: i32,
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
        if (value & 0x03) != 0 {
            todo!("Mode 7 flipping");
        }

        self.transparency_fill = (value & 0xc0) == 0x80;
        self.tile_zero_fill = (value & 0xc0) == 0xc0;
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

fn sign_extend_13(value: u16) -> i32 {
    ((value & 0x1fff).wrapping_sub((value & 0x1000) << 1) as i16) as i32
}

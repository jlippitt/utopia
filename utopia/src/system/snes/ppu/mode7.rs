use tracing::debug;

pub struct Mode7Settings {
    matrix_a: i32,
    matrix_b: i32,
    matrix_c: i32,
    matrix_d: i32,
    write_buffer: u8,
}

impl Mode7Settings {
    pub fn new() -> Self {
        Self {
            matrix_a: 0,
            matrix_b: 0,
            matrix_c: 0,
            matrix_d: 0,
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

    fn word_value(&mut self, value: u8) -> u16 {
        let word_value = u16::from_le_bytes([self.write_buffer, value]);
        self.write_buffer = value;
        word_value
    }
}

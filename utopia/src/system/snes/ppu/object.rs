use tracing::debug;

pub const SIZE_X: [[u16; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

pub const SIZE_Y: [[u16; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [32, 64],
    [32, 32],
];

pub struct ObjectLayer {
    name_base: u16,
    name_offset: u16,
    size_x: [u16; 2],
    size_y: [u16; 2],
}

impl ObjectLayer {
    pub fn new() -> Self {
        Self {
            name_base: 0,
            name_offset: 0,
            size_x: SIZE_X[0],
            size_y: SIZE_Y[0],
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.name_base = (value as u16 & 0x07) << 13;
        self.name_offset = (((value as u16 & 0x18) >> 3) + 1) << 12;

        let size_index = (value >> 5) as usize;
        self.size_x = SIZE_X[size_index];
        self.size_y = SIZE_Y[size_index];

        debug!("OBJ Name Base: {:04X}", self.name_base);
        debug!("OBJ CHR Offset: {:04X}", self.name_offset);
        debug!(
            "OBJ Size: {}x{}, {}x{}",
            self.size_x[0], self.size_y[0], self.size_x[1], self.size_y[1]
        )
    }
}

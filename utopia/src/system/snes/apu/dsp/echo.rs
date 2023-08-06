use tracing::debug;

pub struct Echo {
    volume_left: i32,
    volume_right: i32,
    feedback_volume: i32,
    base_address: u16,
    buffer_size: u16,
    fir_values: [i32; 8],
}

impl Echo {
    pub fn new() -> Self {
        Self {
            volume_left: 0,
            volume_right: 0,
            feedback_volume: 0,
            base_address: 0,
            buffer_size: 0,
            fir_values: [0; 8],
        }
    }

    pub fn set_volume_left(&mut self, value: u8) {
        self.volume_left = value as i8 as i32;
        debug!("Echo Volume Left: {}", self.volume_left);
    }

    pub fn set_volume_right(&mut self, value: u8) {
        self.volume_right = value as i8 as i32;
        debug!("Echo Volume Right: {}", self.volume_right);
    }

    pub fn set_feedback_volume(&mut self, value: u8) {
        self.feedback_volume = value as i8 as i32;
        debug!("Echo Feedback Volume: {}", self.feedback_volume);
    }

    pub fn set_base_address(&mut self, value: u8) {
        self.base_address = (value as u16) << 8;
        debug!("Echo Base Address: {:04X}", self.base_address);
    }

    pub fn set_buffer_size(&mut self, value: u8) {
        self.buffer_size = if value != 0 { (value as u16) * 2048 } else { 4 };
        debug!("Echo Buffer Size: {}", self.buffer_size);
    }

    pub fn set_fir_value(&mut self, index: usize, value: u8) {
        let value = value as i8 as i32;
        self.fir_values[index] = value;
        debug!("Echo FIR {}: {}", index, value);
    }
}

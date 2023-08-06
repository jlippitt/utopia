use crate::util::MirrorVec;
use tracing::debug;

struct RingBuffer {
    base_address: u16,
    size: u16,
    read_index: u16,
    remaining: u16,
}

pub struct Echo {
    volume_left: i32,
    volume_right: i32,
    feedback_volume: i32,
    ring_buffer: RingBuffer,
    write_enabled: bool,
    fir_values: [i32; 8],
    output_buffer: [[i32; 8]; 2],
    write_index: usize,
}

impl Echo {
    pub fn new() -> Self {
        Self {
            volume_left: 0,
            volume_right: 0,
            feedback_volume: 0,
            ring_buffer: RingBuffer::new(),
            write_enabled: false,
            fir_values: [0; 8],
            output_buffer: [[0; 8]; 2],
            write_index: 0,
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
        self.ring_buffer.base_address = (value as u16) << 8;
        debug!(
            "Echo Buffer Base Address: {:04X}",
            self.ring_buffer.base_address
        );
    }

    pub fn set_buffer_size(&mut self, value: u8) {
        self.ring_buffer.size = if value != 0 {
            (value as u16 & 0x0f) * 2048
        } else {
            4
        };
        debug!("Echo Buffer Size: {}", self.ring_buffer.size);
        self.ring_buffer.remaining = self.ring_buffer.size;
    }

    pub fn set_write_enabled(&mut self, write_enabled: bool) {
        self.write_enabled = write_enabled;
        debug!("Echo Write Enabled: {}", self.write_enabled);
    }

    pub fn set_fir_value(&mut self, index: usize, value: u8) {
        let value = value as i8 as i32;
        self.fir_values[index] = value;
        debug!("Echo FIR {}: {}", index, value);
    }

    pub fn step(&mut self, ram: &mut MirrorVec<u8>, input: (i32, i32)) -> (i32, i32) {
        let entry = self.ring_buffer.read_entry(ram);
        let mut output = [0, 0];

        for channel_index in [0, 1] {
            let buffer = &mut self.output_buffer[channel_index];
            buffer[self.write_index] = entry[channel_index];

            let sum = self
                .fir_values
                .iter()
                .enumerate()
                .fold(0, |acc, (fir_index, fir_value)| {
                    let write_index = self.write_index.wrapping_add(fir_index).wrapping_sub(7);
                    let sample = buffer[write_index & 7];
                    acc + ((sample * fir_value) >> 6)
                });

            output[channel_index] = sum.clamp(i32::MIN, i32::MAX);
        }

        if self.write_enabled {
            let feedback = [
                input.0 + ((output[0] * self.feedback_volume) >> 7),
                input.1 + ((output[1] * self.feedback_volume) >> 7),
            ];

            self.ring_buffer.write_entry(ram, feedback);
        }

        self.write_index = (self.write_index + 1) & 7;

        self.ring_buffer.increment_address();

        let left = (output[0] * self.volume_left) >> 7;
        let right = (output[1] * self.volume_right) >> 7;

        (left, right)
    }
}

impl RingBuffer {
    fn new() -> Self {
        Self {
            base_address: 0,
            size: 4,
            read_index: 0,
            remaining: 4,
        }
    }

    fn read_entry(&self, ram: &MirrorVec<u8>) -> [i32; 2] {
        let left = (self.read_word(ram, 0) as i16 as i32) >> 1;
        let right = (self.read_word(ram, 2) as i16 as i32) >> 1;
        [left, right]
    }

    fn read_word(&self, ram: &MirrorVec<u8>, offset: u16) -> u16 {
        let low = self.read_byte(ram, offset);
        let high = self.read_byte(ram, offset.wrapping_add(1));
        u16::from_le_bytes([low, high])
    }

    fn read_byte(&self, ram: &MirrorVec<u8>, offset: u16) -> u8 {
        let address = self
            .base_address
            .wrapping_add(self.read_index)
            .wrapping_add(offset);

        let value = ram[address as usize];
        debug!("Echo Buffer Read: {:04X} => {:02X}", address, value);
        value
    }

    fn write_entry(&mut self, ram: &mut MirrorVec<u8>, entry: [i32; 2]) {
        self.write_word(ram, 0, entry[0] as u16 & 0xfffe);
        self.write_word(ram, 2, entry[1] as u16 & 0xfffe);
    }

    fn write_word(&mut self, ram: &mut MirrorVec<u8>, offset: u16, value: u16) {
        self.write_byte(ram, offset, value as u8);
        self.write_byte(ram, offset.wrapping_add(1), (value >> 8) as u8);
    }

    fn write_byte(&mut self, ram: &mut MirrorVec<u8>, offset: u16, value: u8) {
        let address = self
            .base_address
            .wrapping_add(self.read_index)
            .wrapping_add(offset);

        ram[address as usize] = value;
        debug!("Echo Buffer Write: {:04X} <= {:02X}", address, value);
    }

    fn increment_address(&mut self) {
        self.remaining -= 4;

        if self.remaining == 0 {
            self.read_index = 0;
            self.remaining = self.size;
        } else {
            self.read_index = self.read_index.wrapping_add(4);
        }
    }
}

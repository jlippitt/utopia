use crate::util::MirrorVec;
use tracing::debug;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Normal,
    EndMute,
    EndLoop,
}

pub struct BrrDecoder {
    read_address: u16,
    write_index: usize,
    mode: Mode,
    buffer: [i32; 32],
}

impl BrrDecoder {
    pub fn new() -> Self {
        Self {
            read_address: 0,
            write_index: 0,
            mode: Mode::Normal,
            buffer: [0; 32],
        }
    }

    pub fn restart(&mut self, ram: &MirrorVec<u8>, start_address: u16) {
        self.read_address = start_address;
        self.decode_next(ram);
    }

    fn decode_next(&mut self, ram: &MirrorVec<u8>) {
        let header = self.next_byte(ram);
        debug!("Block Header: {:02X}", header);

        let filter = (header >> 2) & 3;
        let shift = header >> 4;

        for _ in 0..8 {
            let byte = self.next_byte(ram);
            debug!("{:02}", byte);
            self.filter_sample(filter, shift, byte >> 4);
            self.filter_sample(filter, shift, byte & 0x0f);
        }

        self.mode = match header & 3 {
            1 => Mode::EndMute,
            3 => Mode::EndLoop,
            _ => Mode::Normal,
        };

        debug!("Block Mode: {:?}", self.mode);
    }

    fn filter_sample(&mut self, filter: u8, shift: u8, nibble: u8) {
        // Sign the 4-bit value (-8 to +7)
        let sample = (nibble as i32) - ((nibble as i32 & 0x08) << 1);

        let input = if shift <= 12 {
            (sample << shift) >> 1
        } else {
            ((sample >> 3) << 12) >> 1
        };

        let older = self.buffer[self.write_index.wrapping_sub(2) & 31];
        let old = self.buffer[self.write_index.wrapping_sub(1) & 31];

        let output = match filter {
            0 => input,
            1 => input + old + (-old >> 4),
            2 => input + (old * 2) + ((-old * 3) >> 5) - older + (older >> 4),
            3 => input + (old * 2) + ((-old * 13) >> 6) - older + ((older * 3) >> 4),
            _ => unreachable!(),
        };

        let clamped_output = output.clamp(i16::MIN as i32, i16::MAX as i32);
        self.buffer[self.write_index] = clamped_output;
        debug!("Sample {}: {}", self.write_index, clamped_output);
        self.write_index = (self.write_index + 1) & 31;
    }

    fn next_byte(&mut self, ram: &MirrorVec<u8>) -> u8 {
        let value = ram[self.read_address as usize];
        debug!("BRR Read: {:04X} => {:02X}", self.read_address, value);
        self.read_address = self.read_address.wrapping_add(1);
        value
    }
}

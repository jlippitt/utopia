use tracing::warn;

const TOTAL_LINES: u32 = 154;
const DOTS_PER_LINE: u64 = 456;

pub struct Ppu {
    line: u32,
    dot: u64,
}

impl Ppu {
    pub fn new() -> Self {
        Self { line: 0, dot: 0 }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn dot(&self) -> u64 {
        self.dot
    }

    pub fn read(&self, address: u8) -> u8 {
        match address {
            0x44 => self.line as u8,
            _ => panic!("PPU register read {:02X} not yet implemented", address),
        }
    }

    pub fn write(&self, address: u8, _value: u8) {
        match address {
            _ => warn!("PPU register write {:02X} not yet implemented", address),
        }
    }

    pub fn step(&mut self, cycles: u64) {
        self.dot += cycles;

        if self.dot == DOTS_PER_LINE {
            self.dot = 0;
            self.line += 1;

            if self.line == TOTAL_LINES {
                self.line = 0;
            }
        }
    }
}

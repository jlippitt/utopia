use super::interrupt::{Interrupt, InterruptType};
use tracing::{debug, warn};

const VBLANK_LINE: u32 = 144;
const TOTAL_LINES: u32 = 154;

const DOTS_PER_LINE: u64 = 456;

struct Control {
    raw: u8,
}

pub struct Ppu {
    ready: bool,
    line: u32,
    dot: u64,
    control: Control,
    scroll_y: u8,
    scroll_x: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ready: false,
            line: 0,
            dot: 0,
            control: Control { raw: 0 },
            scroll_y: 0,
            scroll_x: 0,
        }
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn start_frame(&mut self) {
        self.ready = false;
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn dot(&self) -> u64 {
        self.dot
    }

    pub fn read(&self, address: u8) -> u8 {
        match address {
            0x40 => self.control.raw,
            0x42 => self.scroll_y,
            0x43 => self.scroll_x,
            0x44 => self.line as u8,
            _ => panic!("PPU register read {:02X} not yet implemented", address),
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x40 => self.control.raw = value,
            0x42 => {
                self.scroll_y = value;
                debug!("PPU Scroll Y: {}", self.scroll_y);
            }
            0x43 => {
                self.scroll_x = value;
                debug!("PPU Scroll X: {}", self.scroll_x);
            }
            _ => warn!("PPU register write {:02X} not yet implemented", address),
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt, cycles: u64) {
        self.dot += cycles;

        if self.dot == DOTS_PER_LINE {
            self.dot = 0;
            self.line += 1;

            if self.line == TOTAL_LINES {
                self.line = 0;
            } else if self.line == VBLANK_LINE {
                self.ready = true;
                interrupt.trigger(InterruptType::VBlank);
            }
        }
    }
}

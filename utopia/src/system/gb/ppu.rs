use super::interrupt::{Interrupt, InterruptType};
use crate::util::MirrorVec;
use tracing::{debug, warn};

const VRAM_SIZE: usize = 8192;

const VBLANK_LINE: u32 = 144;
const TOTAL_LINES: u32 = 154;

const DOTS_PER_LINE: u64 = 456;

struct Control {
    lcd_enable: bool,
    raw: u8,
}

pub struct Ppu {
    ready: bool,
    line: u32,
    dot: u64,
    control: Control,
    scroll_y: u8,
    scroll_x: u8,
    vram: MirrorVec<u8>,
    oam: [u8; 160],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ready: false,
            line: 0,
            dot: 0,
            control: Control {
                lcd_enable: false,
                raw: 0,
            },
            scroll_y: 0,
            scroll_x: 0,
            vram: MirrorVec::new(VRAM_SIZE),
            oam: [0; 160],
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

    pub fn read_register(&self, address: u8) -> u8 {
        match address {
            0x40 => self.control.raw,
            0x42 => self.scroll_y,
            0x43 => self.scroll_x,
            0x44 => self.line as u8,
            _ => panic!("PPU register read {:02X} not yet implemented", address),
        }
    }

    pub fn write_register(&mut self, address: u8, value: u8) {
        match address {
            0x40 => {
                self.control.lcd_enable = (value & 0x80) != 0;

                if !self.control.lcd_enable {
                    self.line = 0;
                    self.dot = 0;
                }

                self.control.raw = value;

                debug!("LCD Enable: {}", self.control.lcd_enable);
            }
            0x42 => {
                self.scroll_y = value;
                debug!("Scroll Y: {}", self.scroll_y);
            }
            0x43 => {
                self.scroll_x = value;
                debug!("Scroll X: {}", self.scroll_x);
            }
            _ => warn!("PPU register write {:02X} not yet implemented", address),
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[address as usize] = value;
    }

    pub fn read_oam(&self, address: u8) -> u8 {
        self.oam[address as usize]
    }

    pub fn write_oam(&mut self, address: u8, value: u8) {
        self.oam[address as usize] = value;
    }

    pub fn step(&mut self, interrupt: &mut Interrupt, cycles: u64) {
        if !self.control.lcd_enable {
            return;
        }

        self.dot += cycles;

        if self.dot == DOTS_PER_LINE {
            self.dot = 0;
            self.line += 1;

            if self.line == TOTAL_LINES {
                self.line = 0;
            } else if self.line == VBLANK_LINE {
                self.ready = true;
                interrupt.raise(InterruptType::VBlank);
            }
        }
    }
}

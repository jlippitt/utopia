use tracing::{debug, warn};

const VBLANK_LINE: u32 = 241;
const PRE_RENDER_LINE: u32 = 241;
const TOTAL_LINES: u32 = 262;

const DOTS_PER_LINE: u32 = 341;

pub struct Ppu {
    line: u32,
    dot: u32,
    nmi_occurred: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            line: 0,
            dot: 0,
            nmi_occurred: false,
        }
    }

    pub fn v_counter(&self) -> u32 {
        self.line
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address & 7 {
            2 => {
                // TODO: Open bus
                let mut result: u8 = 0;

                if self.nmi_occurred {
                    self.nmi_occurred = false;
                    debug!("NMI Occurred: {}", self.nmi_occurred);
                    result |= 0x80;
                }

                result
            }
            _ => panic!("PPU read {:04X} not yet implemented", address),
        }
    }

    pub fn write(&mut self, address: u16, _value: u8) {
        match address & 7 {
            _ => warn!("PPU write {:04X} not yet implemented", address),
        }
    }

    pub fn step(&mut self) {
        // Extremely simple state machine for now
        self.dot += 1;

        if self.dot == DOTS_PER_LINE {
            self.dot = 0;
            self.line += 1;

            if self.line == VBLANK_LINE {
                self.nmi_occurred = true;
                debug!("NMI Occurred: {}", self.nmi_occurred);
            } else if self.line == PRE_RENDER_LINE {
                self.nmi_occurred = false;
                debug!("NMI Occurred: {}", self.nmi_occurred);
            } else if self.line == TOTAL_LINES {
                self.line = 0;
            }
        }
    }
}

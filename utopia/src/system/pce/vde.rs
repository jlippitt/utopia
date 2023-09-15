use super::vdc::{Vdc, VdcInterrupt};
use tracing::{debug, warn};

const CYCLES_PER_LINE: u64 = 1364;

const LINES_PER_FRAME_NORMAL: u16 = 262;
const LINES_PER_FRAME_INTERLACE: u16 = 263;

pub struct Vde {
    line_cycles: u64,
    line_counter: u16,
    lines_per_frame: u16,
}

impl Vde {
    pub fn new() -> Self {
        Self {
            line_cycles: 0,
            line_counter: 0,
            lines_per_frame: LINES_PER_FRAME_NORMAL,
        }
    }

    pub fn line_counter(&self) -> u16 {
        self.line_counter
    }

    pub fn read(&self, address: u16, _prev_value: u8) -> u8 {
        unimplemented!("VDE Read: {:04X}", address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 7 {
            0 => {
                // TODO: Dot clock
                // TODO: Color burst bit

                self.lines_per_frame = if (value & 0x02) != 0 {
                    LINES_PER_FRAME_INTERLACE
                } else {
                    LINES_PER_FRAME_NORMAL
                };

                debug!("VDE Lines Per Frame: {:04X}", self.lines_per_frame);
            }
            _ => warn!("Unimplemented VDE Write: {:04X} <= {:02X}", address, value),
        }
    }

    pub fn step(&mut self, vdc: &mut Vdc, cycles: u64) {
        self.line_cycles += cycles;

        if self.line_cycles >= CYCLES_PER_LINE {
            self.line_cycles -= CYCLES_PER_LINE;
            self.line_counter += 1;

            if self.line_counter == self.lines_per_frame {
                self.line_counter = 0;
                vdc.clear_interrupt(VdcInterrupt::VBLANK);
            } else if (self.line_counter + 64) == vdc.display_height() {
                vdc.raise_interrupt(VdcInterrupt::VBLANK);
            }
        }
    }
}

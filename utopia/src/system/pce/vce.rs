use super::vdc::Vdc;
use bitfield_struct::bitfield;
use tracing::{debug, warn};

const CYCLES_PER_LINE: u64 = 1364;

const LINES_PER_FRAME_NORMAL: u16 = 262;
const LINES_PER_FRAME_INTERLACE: u16 = 263;

#[bitfield(u16)]
pub struct Color {
    #[bits(3)]
    pub blue: u8,
    #[bits(3)]
    pub red: u8,
    #[bits(3)]
    pub green: u8,
    #[bits(7)]
    __: u8,
}

pub struct Vce {
    frame_done: bool,
    line_cycles: u64,
    hblank_start: u64,
    line_counter: u16,
    lines_per_frame: u16,
    cycles_per_dot: u64,
    palette_index: u16,
    palette: [Color; 512],
}

impl Vce {
    pub fn new() -> Self {
        Self {
            frame_done: false,
            line_cycles: 0,
            hblank_start: u64::MAX,
            line_counter: 0,
            lines_per_frame: LINES_PER_FRAME_NORMAL,
            cycles_per_dot: 4,
            palette_index: 0,
            palette: [Color::new(); 512],
        }
    }

    pub fn frame_done(&self) -> bool {
        self.frame_done
    }

    pub fn start_frame(&mut self) {
        self.frame_done = false;
    }

    pub fn line_counter(&self) -> u16 {
        self.line_counter
    }

    pub fn read(&self, address: u16, _prev_value: u8) -> u8 {
        unimplemented!("VCE Read: {:04X}", address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 7 {
            0 => {
                self.cycles_per_dot = match value & 3 {
                    0 => 4,
                    1 => 3,
                    _ => 2,
                };

                // TODO: Color burst bit

                self.lines_per_frame = if (value & 0x02) != 0 {
                    LINES_PER_FRAME_INTERLACE
                } else {
                    LINES_PER_FRAME_NORMAL
                };

                debug!("VCE Lines Per Frame: {:04X}", self.lines_per_frame);
            }
            2 => {
                self.palette_index = (self.palette_index & 0xff00) | value as u16;
                debug!("VCE Palette Index: {}", self.palette_index);
            }
            3 => {
                self.palette_index = (self.palette_index & 0xff) | ((value as u16 & 0x01) << 8);
                debug!("VCE Palette Index: {}", self.palette_index);
            }
            4 => {
                let color = &mut self.palette[self.palette_index as usize];

                *color = ((u16::from(*color) & 0xff00) | value as u16).into();

                debug!(
                    "VCE Palette Write (Low): {:04X} <= {:02X} ({:04X})",
                    self.palette_index,
                    value,
                    u16::from(*color),
                );
            }
            5 => {
                let color = &mut self.palette[self.palette_index as usize];

                *color = ((u16::from(*color) & 0xff) | ((value as u16 & 0x01) << 8)).into();

                debug!(
                    "VCE Palette Write (High): {:04X} <= {:02X} ({:04X})",
                    self.palette_index,
                    value,
                    u16::from(*color),
                );

                self.palette_index = self.palette_index.wrapping_add(1) & 511;
            }
            _ => warn!("Unimplemented VDE Write: {:04X} <= {:02X}", address, value),
        }
    }

    pub fn step(&mut self, vdc: &mut Vdc, cycles: u64) {
        self.line_cycles += cycles;

        if self.line_cycles >= self.hblank_start {
            self.hblank_start = u64::MAX;
            vdc.render_line(&self.palette, self.line_counter);
        }

        if self.line_cycles >= CYCLES_PER_LINE {
            self.line_cycles -= CYCLES_PER_LINE;
            self.line_counter += 1;

            if self.line_counter == self.lines_per_frame {
                self.line_counter = 0;
                vdc.on_frame_start();
            } else if self.line_counter == vdc.display_height() {
                self.frame_done = true;
                vdc.on_vblank_start();
            }

            if (self.line_counter + 64) == vdc.scanline_match() {
                vdc.on_scanline_match();
            }

            self.hblank_start = if self.line_counter < vdc.display_height() {
                vdc.display_width() as u64 * self.cycles_per_dot
            } else {
                u64::MAX
            };
        }
    }
}

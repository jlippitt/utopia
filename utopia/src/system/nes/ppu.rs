use crate::core::mos6502::{Interrupt, INT_NMI};
use tracing::{debug, warn};

const VBLANK_LINE: u32 = 241;
const PRE_RENDER_LINE: u32 = 241;
const TOTAL_LINES: u32 = 262;

const DOTS_PER_LINE: u32 = 341;

struct Registers {
    v: u16,
    t: u16,
    x: u8,
    w: bool,
}

pub struct Ppu {
    line: u32,
    dot: u32,
    nmi_occurred: bool,
    nmi_active: bool,
    regs: Registers,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            line: 0,
            dot: 0,
            nmi_occurred: false,
            nmi_active: false,
            regs: Registers {
                v: 0,
                t: 0,
                x: 0,
                w: false,
            }
        }
    }

    pub fn v_counter(&self) -> u32 {
        self.line
    }

    pub fn read(&mut self, interrupt: &mut Interrupt, address: u16) -> u8 {
        match address & 7 {
            2 => {
                // TODO: Open bus
                let mut result: u8 = 0;

                if self.nmi_occurred {
                    result |= 0x80;
                    self.nmi_occurred = false;
                    debug!("NMI Occurred: {}", self.nmi_occurred);
                    *interrupt &= !INT_NMI;
                }

                self.regs.w = false;

                result
            }
            _ => panic!("PPU read {:04X} not yet implemented", address),
        }
    }

    pub fn write(&mut self, interrupt: &mut Interrupt, address: u16, value: u8) {
        match address & 7 {
            0 => {
                let nmi_active = (value & 0x80) != 0;

                if !nmi_active {
                    *interrupt &= !INT_NMI;
                } else if self.nmi_occurred && !self.nmi_active {
                    *interrupt |= INT_NMI;
                }

                self.nmi_active = nmi_active;
                debug!("NMI Active: {}", self.nmi_active);

                self.regs.t = (self.regs.t & 0x73ff) | ((value as u16 & 0x03) << 10);
                debug!("PPU TMP Address: {:04X}", self.regs.t);
            }
            5 => {
                if self.regs.w {
                    self.regs.t = (self.regs.t & 0x0c1f) | ((value as u16 & 0xf8) << 2) | ((value as u16 & 0x07) << 12);
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                } else {
                    self.regs.t = (self.regs.t & 0x7fe0) | ((value >> 3) as u16);
                    self.regs.x = value & 0x07;
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                    debug!("PPU Fine X: {}", self.regs.x);
                }

                self.regs.w = !self.regs.w;
            },
            6 => {
                if self.regs.w {
                    self.regs.t = (self.regs.t & 0xff00) | value as u16;
                    self.regs.v = self.regs.t;
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                    debug!("PPU VRAM Address: {:04X}", self.regs.v);
                } else {
                    self.regs.t = (self.regs.t & 0xff) | ((value as u16 & 0x3f) << 8);
                    debug!("PPU TMP Address: {:04X}", self.regs.t);
                }

                self.regs.w = !self.regs.w;
            }
            _ => warn!("PPU write {:04X} not yet implemented", address),
        }
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {
        // Extremely simple state machine for now
        self.dot += 1;

        if self.dot == DOTS_PER_LINE {
            self.dot = 0;
            self.line += 1;

            if self.line == VBLANK_LINE {
                self.nmi_occurred = true;
                debug!("NMI Occurred: {}", self.nmi_occurred);

                if self.nmi_active {
                    *interrupt |= INT_NMI;
                }
            } else if self.line == PRE_RENDER_LINE {
                self.nmi_occurred = false;
                *interrupt &= !INT_NMI;
                debug!("NMI Occurred: {}", self.nmi_occurred);
            } else if self.line == TOTAL_LINES {
                self.line = 0;
            }
        }
    }
}

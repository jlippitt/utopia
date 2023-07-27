use crate::core::wdc65c816::{Interrupt, INT_NMI};
use std::fmt;
use std::iter::Peekable;
use std::slice;
use tracing::{debug, warn};

pub const FAST_CYCLES: u64 = 6;
pub const SLOW_CYCLES: u64 = 8;
pub const EXTRA_SLOW_CYCLES: u64 = 12;

// TODO: Interlace
pub const VBLANK_LINE_NORMAL: u16 = 225;
pub const VBLANK_LINE_OVERSCAN: u16 = 240;
pub const TOTAL_LINES: u16 = 262;

pub const CYCLES_PER_LINE: u64 = 1364;

pub const TIMER_IRQ: Interrupt = 0x04;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum IrqMode {
    None,
    H,
    V,
    HV,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Event {
    HBlank,
    NewLine,
    Irq,
}

#[derive(Copy, Clone, Debug)]
pub struct LineEvent {
    cycles: u64,
    event: Event,
}

const LINE_EVENTS: [LineEvent; 2] = [
    LineEvent {
        cycles: 1112,
        event: Event::HBlank,
    },
    LineEvent {
        cycles: CYCLES_PER_LINE,
        event: Event::NewLine,
    },
];

fn irq_event(cycles: u64) -> LineEvent {
    LineEvent {
        cycles,
        event: Event::Irq,
    }
}

pub struct Clock {
    line_cycles: u64,
    banked_cycles: u64,
    fast_rom_cycles: u64,
    fast_rom_supported: bool,
    next_event: LineEvent,
    line_events: Peekable<slice::Iter<'static, LineEvent>>,
    line: u16,
    vblank_line: u16,
    frame: u64,
    nmi_occurred: bool,
    nmi_active: bool,
    irq_mode: IrqMode,
    irq_x: u16,
    irq_y: u16,
    irq_cycle: Option<u64>,
}

impl Clock {
    pub fn new(fast_rom_supported: bool) -> Self {
        let mut line_events = LINE_EVENTS.iter().peekable();
        let next_event = line_events.next().unwrap();

        Self {
            line_cycles: 0,
            banked_cycles: 0,
            fast_rom_cycles: SLOW_CYCLES,
            fast_rom_supported,
            next_event: *next_event,
            line_events,
            line: 0,
            vblank_line: VBLANK_LINE_NORMAL,
            frame: 0,
            nmi_occurred: false,
            nmi_active: false,
            irq_mode: IrqMode::None,
            irq_x: 0x01ff,
            irq_y: 0x01ff,
            irq_cycle: None,
        }
    }

    pub fn cycles(&self) -> u64 {
        self.banked_cycles + self.line_cycles
    }

    pub fn line(&self) -> u16 {
        self.line
    }

    pub fn dot(&self) -> u64 {
        // TODO: 'Fat' dots
        self.line_cycles >> 2
    }

    pub fn odd_frame(&self) -> bool {
        (self.frame & 1) != 0
    }

    pub fn vblank_line(&self) -> u16 {
        self.vblank_line
    }

    pub fn nmi_occurred(&self) -> bool {
        self.nmi_occurred
    }

    pub fn set_overscan(&mut self, enabled: bool) {
        self.vblank_line = if enabled {
            VBLANK_LINE_OVERSCAN
        } else {
            VBLANK_LINE_NORMAL
        };
        debug!("VBlank Line: {}", self.vblank_line);
    }

    pub fn set_nmi_occurred(&mut self, interrupt: &mut Interrupt, nmi_occurred: bool) {
        if nmi_occurred && self.nmi_active {
            *interrupt |= INT_NMI;
        } else {
            *interrupt &= !INT_NMI;
        }

        self.nmi_occurred = nmi_occurred;
        debug!("NMI Occurred: {}", self.nmi_occurred);
    }

    pub fn set_nmi_active(&mut self, interrupt: &mut Interrupt, nmi_active: bool) {
        if nmi_active && !self.nmi_active && self.nmi_occurred {
            *interrupt |= INT_NMI;
        } else if !nmi_active {
            *interrupt &= !INT_NMI;
        }

        self.nmi_active = nmi_active;
        debug!("NMI Active: {}", self.nmi_active);
    }

    pub fn set_irq_mode(&mut self, interrupt: &mut Interrupt, value: u8) {
        self.irq_mode = match value {
            0 => IrqMode::None,
            1 => IrqMode::H,
            2 => IrqMode::V,
            3 => IrqMode::HV,
            _ => panic!("Invalid IRQ mode: {}", value),
        };

        debug!("IRQ Mode: {:?}", self.irq_mode);

        if self.irq_mode == IrqMode::None {
            *interrupt &= !TIMER_IRQ;
        }

        self.update_irq_cycle(false);
    }

    pub fn set_irq_x_low(&mut self, value: u8) {
        self.irq_x = (self.irq_x & 0xff00) | (value as u16);
        debug!("IRQ X: {}", self.irq_x);
        self.update_irq_cycle(false);
    }

    pub fn set_irq_x_high(&mut self, value: u8) {
        self.irq_x = (self.irq_x & 0xff) | ((value as u16 & 0x01) << 8);
        debug!("IRQ X: {}", self.irq_x);
        self.update_irq_cycle(false);
    }

    pub fn set_irq_y_low(&mut self, value: u8) {
        self.irq_y = (self.irq_y & 0xff00) | (value as u16);
        debug!("IRQ Y: {}", self.irq_y);
        self.update_irq_cycle(false);
    }

    pub fn set_irq_y_high(&mut self, value: u8) {
        self.irq_y = (self.irq_y & 0xff) | ((value as u16 & 0x01) << 8);
        debug!("IRQ Y: {}", self.irq_y);
        self.update_irq_cycle(false);
    }

    pub fn set_fast_rom_enabled(&mut self, enabled: bool) {
        if enabled && !self.fast_rom_supported {
            warn!("Attempted to enable FastROM, but it is not supported by this cartridge type");
            return;
        }

        self.fast_rom_cycles = if enabled { FAST_CYCLES } else { SLOW_CYCLES };
        debug!("FastROM Cycles: {}", self.fast_rom_cycles);
    }

    pub fn add_cycles(&mut self, cycles: u64) {
        self.line_cycles += cycles;
    }

    pub fn event(&mut self) -> Option<Event> {
        let current_event = self.next_event;

        if self.line_cycles >= current_event.cycles {
            self.next_event = self.next_event().unwrap_or_else(|| {
                self.banked_cycles += CYCLES_PER_LINE;
                self.line_cycles -= CYCLES_PER_LINE;
                self.line += 1;

                if self.line == TOTAL_LINES {
                    self.line = 0;
                    self.frame += 1;
                }

                self.update_irq_cycle(true);

                self.line_events = LINE_EVENTS.iter().peekable();
                self.next_event().unwrap()
            });

            Some(current_event.event)
        } else {
            None
        }
    }

    pub fn cycles_for_address(&self, address: u32) -> u64 {
        if (address & 0x408000) != 0 {
            return if (address & 0x800000) != 0 {
                self.fast_rom_cycles
            } else {
                SLOW_CYCLES
            };
        }

        if (address.wrapping_add(0x6000) & 0x4000) != 0 {
            return SLOW_CYCLES;
        }

        if (address.wrapping_sub(0x4000) & 0x7e00) != 0 {
            return FAST_CYCLES;
        }

        return EXTRA_SLOW_CYCLES;
    }

    fn next_event(&mut self) -> Option<LineEvent> {
        if let Some(&next_event) = self.line_events.peek() {
            if let Some(irq_cycle) = self.irq_cycle {
                if irq_cycle < next_event.cycles {
                    self.irq_cycle = None;
                    return Some(irq_event(irq_cycle));
                }
            }

            self.line_events.next();
            return Some(*next_event);
        }

        if let Some(irq_cycle) = self.irq_cycle {
            self.irq_cycle = None;
            return Some(irq_event(irq_cycle));
        }

        None
    }

    fn update_irq_cycle(&mut self, end_of_line: bool) {
        self.irq_cycle = match self.irq_mode {
            IrqMode::None => None,
            IrqMode::V => {
                if self.line == self.irq_y {
                    Some(0)
                } else {
                    None
                }
            }
            IrqMode::H => Some((self.irq_x as u64) << 2),
            IrqMode::HV => {
                if self.line == self.irq_y {
                    Some((self.irq_x as u64) << 2)
                } else {
                    None
                }
            }
        };

        // During mid-line updates, don't schedule an IRQ if we've already
        // passed the cycle it would be on
        if !end_of_line && self.irq_cycle >= Some(self.line_cycles) {
            self.irq_cycle = None;
        }
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "F={} V={} H={} T={}",
            self.frame,
            self.line(),
            self.dot(),
            self.cycles()
        )
    }
}

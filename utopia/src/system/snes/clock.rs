use std::fmt;
use std::iter::Peekable;
use std::slice;
use tracing::debug;

pub const FAST_CYCLES: u64 = 6;
pub const SLOW_CYCLES: u64 = 8;
pub const EXTRA_SLOW_CYCLES: u64 = 12;

// TODO: Interlace
pub const TOTAL_LINES: u16 = 262;

pub const CYCLES_PER_LINE: u64 = 1364;

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
    line: u16,
    fast_rom_cycles: u64,
    next_event: LineEvent,
    line_events: Peekable<slice::Iter<'static, LineEvent>>,
    irq_cycle: Option<u64>,
    frame: u64,
}

impl Clock {
    pub fn new() -> Self {
        let mut line_events = LINE_EVENTS.iter().peekable();
        let next_event = line_events.next().unwrap();

        Self {
            line_cycles: 0,
            banked_cycles: 0,
            line: 0,
            fast_rom_cycles: SLOW_CYCLES,
            next_event: *next_event,
            line_events,
            irq_cycle: None,
            frame: 0,
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

    pub fn set_irq_cycle(&mut self, value: Option<u64>) {
        self.irq_cycle = value;

        if let Some(irq_cycle) = self.irq_cycle {
            debug!("IRQ Cycle: {}", irq_cycle);
        } else {
            debug!("IRQ Cycle: None");
        }
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

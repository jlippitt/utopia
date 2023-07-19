use std::fmt;
use std::slice;

pub const FAST_CYCLES: u64 = 6;
pub const SLOW_CYCLES: u64 = 8;
pub const EXTRA_SLOW_CYCLES: u64 = 12;

// TODO: Interlace/Overscan
pub const TOTAL_LINES: u32 = 262;

pub const CYCLES_PER_LINE: u64 = 1364;

// TODO: More specific line types
const LINE_EVENTS: [(u64, Event); 1] = [(CYCLES_PER_LINE, Event::NewLine)];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Event {
    NewLine,
}

pub struct Clock {
    cycles: u64,
    dot: u64,
    line: u32,
    fast_rom_cycles: u64,
    next_event: (u64, Event),
    line_events: slice::Iter<'static, (u64, Event)>,
}

impl Clock {
    pub fn new() -> Self {
        let mut line_events = LINE_EVENTS.iter();
        let next_event = line_events.next().unwrap();

        Self {
            cycles: 0,
            line: 0,
            dot: 0,
            fast_rom_cycles: SLOW_CYCLES,
            next_event: *next_event,
            line_events,
        }
    }

    pub fn cycles(&self) -> u64 {
        self.cycles + self.dot
    }

    pub fn add_cycles(&mut self, cycles: u64) {
        self.dot += cycles;
    }

    pub fn event(&mut self) -> Option<Event> {
        let event = self.next_event;

        if self.dot >= event.0 {
            self.next_event = *self.line_events.next().unwrap_or_else(|| {
                self.cycles += CYCLES_PER_LINE;
                self.dot -= CYCLES_PER_LINE;
                self.line += 1;

                if self.line == TOTAL_LINES {
                    self.line = 0;
                }

                self.line_events = LINE_EVENTS.iter();
                self.line_events.next().unwrap()
            });

            Some(event.1)
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
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Long dots
        write!(f, "V={} H={} T={}", self.line, self.dot >> 2, self.cycles)
    }
}

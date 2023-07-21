use std::fmt;
use std::slice;

pub const FAST_CYCLES: u64 = 6;
pub const SLOW_CYCLES: u64 = 8;
pub const EXTRA_SLOW_CYCLES: u64 = 12;

// TODO: Interlace
pub const TOTAL_LINES: u16 = 262;

pub const CYCLES_PER_LINE: u64 = 1364;

const LINE_EVENTS: [(u64, Event); 2] = [(1112, Event::HBlank), (CYCLES_PER_LINE, Event::NewLine)];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Event {
    HBlank,
    NewLine,
}

pub struct Clock {
    line_cycles: u64,
    banked_cycles: u64,
    line: u16,
    fast_rom_cycles: u64,
    next_event: (u64, Event),
    line_events: slice::Iter<'static, (u64, Event)>,
}

impl Clock {
    pub fn new() -> Self {
        let mut line_events = LINE_EVENTS.iter();
        let next_event = line_events.next().unwrap();

        Self {
            line_cycles: 0,
            banked_cycles: 0,
            line: 0,
            fast_rom_cycles: SLOW_CYCLES,
            next_event: *next_event,
            line_events,
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

    pub fn add_cycles(&mut self, cycles: u64) {
        self.line_cycles += cycles;
    }

    pub fn event(&mut self) -> Option<Event> {
        let event = self.next_event;

        if self.line_cycles >= event.0 {
            self.next_event = *self.line_events.next().unwrap_or_else(|| {
                self.banked_cycles += CYCLES_PER_LINE;
                self.line_cycles -= CYCLES_PER_LINE;
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
        write!(f, "V={} H={} T={}", self.line(), self.dot(), self.cycles())
    }
}

use super::super::Clock;
use tracing::debug;

pub struct Latch {
    enabled: bool,
    latched: bool,
    counter_x: u16,
    counter_y: u16,
    high_byte_x: bool,
    high_byte_y: bool,
}

impl Latch {
    pub fn new() -> Self {
        Self {
            enabled: true,
            latched: false,
            counter_x: 0x01ff,
            counter_y: 0x01ff,
            high_byte_x: false,
            high_byte_y: false,
        }
    }

    pub fn set_enabled(&mut self, clock: &Clock, enabled: bool) {
        let prev_enabled = self.enabled;

        self.enabled = enabled;
        debug!("PPU Latch Enabled: {}", enabled);

        if self.enabled && !prev_enabled {
            self.latch_counters(clock);
        }
    }

    pub fn latch_counters(&mut self, clock: &Clock) {
        if !self.enabled {
            return;
        }

        self.latched = true;
        self.counter_x = clock.dot() as u16;
        self.counter_y = clock.line();
        debug!("PPU Counters Latched: {}", self.latched);
        debug!("PPU Counter X: {}", self.counter_x);
        debug!("PPU Counter Y: {}", self.counter_y);
    }

    pub fn counter_x(&mut self) -> u8 {
        let value = if self.high_byte_x {
            (self.counter_x >> 8) as u8
        } else {
            self.counter_x as u8
        };

        self.high_byte_x = !self.high_byte_x;

        value
    }

    pub fn counter_y(&mut self) -> u8 {
        let value = if self.high_byte_y {
            (self.counter_y >> 8) as u8
        } else {
            self.counter_y as u8
        };

        self.high_byte_y = !self.high_byte_y;

        value
    }

    pub fn poll_status(&mut self) -> bool {
        let latched = self.latched;

        if self.enabled && self.latched {
            self.latched = false;
            debug!("PPU Counters Latched: {}", self.latched);
        }

        self.high_byte_x = false;
        self.high_byte_y = false;

        return latched;
    }
}

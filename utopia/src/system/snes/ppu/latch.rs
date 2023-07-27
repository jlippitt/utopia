use super::super::Clock;
use tracing::debug;

pub struct Latch {
    enabled: bool,
    latched: bool,
    counter: [u16; 2],
    high_byte: [bool; 2],
}

impl Latch {
    pub fn new() -> Self {
        Self {
            enabled: true,
            latched: false,
            counter: [0x01ff; 2],
            high_byte: [false; 2],
        }
    }

    pub fn set_enabled(&mut self, clock: &Clock, enabled: bool) {
        if !enabled && self.enabled {
            self.latch_counters(clock);
        }

        self.enabled = enabled;
        debug!("PPU Latch Enabled: {}", enabled);
    }

    pub fn latch_counters(&mut self, clock: &Clock) {
        if !self.enabled {
            return;
        }

        self.latched = true;
        self.counter[0] = clock.dot() as u16;
        self.counter[1] = clock.line();
        debug!("PPU Counters Latched: {}", self.latched);
        debug!("PPU Counter X: {}", self.counter[0]);
        debug!("PPU Counter Y: {}", self.counter[1]);
    }

    pub fn counter(&mut self, index: usize) -> u8 {
        let value = if self.high_byte[index] {
            (self.counter[index] >> 8) as u8
        } else {
            self.counter[index] as u8
        };

        self.high_byte[index] = !self.high_byte[index];

        value
    }

    pub fn poll_status(&mut self) -> bool {
        let latched = self.latched;

        if self.enabled {
            self.latched = false;
            debug!("PPU Counters Latched: {}", self.latched);
        }

        self.high_byte = [false; 2];

        return latched;
    }
}

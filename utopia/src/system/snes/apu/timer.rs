use tracing::trace;

pub struct Timer {
    enabled: bool,
    counter: u8,
    divider: u8,
    output: u8,
    id: u32,
}

impl Timer {
    pub fn new(id: u32) -> Self {
        Self {
            enabled: false,
            counter: 0,
            divider: 0,
            output: 0,
            id,
        }
    }

    pub fn get_and_reset_output(&mut self) -> u8 {
        let value = self.output;
        self.output = 0;
        trace!("Timer {} Output Reset", self.id);
        value
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && !self.enabled {
            self.counter = 0;
            self.output = 0;
            trace!("Timer {} Counter Reset", self.id);
            trace!("Timer {} Output Reset", self.id);
        }

        self.enabled = enabled;
        trace!("Timer {} Enabled: {}", self.id, self.enabled);
    }

    pub fn set_divider(&mut self, value: u8) {
        self.divider = value;
        trace!("Timer {} Divider: {}", self.id, self.divider);
    }

    pub fn step(&mut self) {
        if !self.enabled {
            return;
        }

        self.counter = self.counter.wrapping_add(1);

        if self.counter == self.divider {
            self.counter = 0;
            self.output = self.output.wrapping_add(1) & 15;
            trace!("Timer {} Output: {}", self.id, self.output);
        }
    }
}

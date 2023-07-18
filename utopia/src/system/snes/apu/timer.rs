use tracing::debug;

pub struct Timer {
    enabled: bool,
    divider: u8,
    counter: u8,
    id: u32,
}

impl Timer {
    pub fn new(id: u32) -> Self {
        Self {
            enabled: false,
            divider: 0,
            counter: 0,
            id,
        }
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("Timer {} Enabled: {}", self.id, self.enabled);
    }

    pub fn set_divider(&mut self, value: u8) {
        self.divider = value;
        debug!("Timer {} Divider: {}", self.id, self.divider);
    }
}

use tracing::debug;

const BIT_NAME: [&str; 2] = ["Main Screen", "Sub-Screen"];

#[derive(Copy, Clone)]
pub struct Toggle {
    enabled: u8,
    name: &'static str,
}

impl Toggle {
    pub fn new(name: &'static str) -> Self {
        Self { enabled: 0, name }
    }

    pub fn any(&self) -> bool {
        self.enabled != 0
    }

    pub fn has(&self, bit: usize) -> bool {
        self.enabled & (1 << bit) != 0
    }

    pub fn set(&mut self, bit: usize, enabled: bool) {
        if enabled {
            self.enabled |= 1 << bit;
        } else {
            self.enabled &= !(1 << bit);
        }

        debug!("{} {} Enabled: {}", self.name, BIT_NAME[bit], enabled);
    }
}

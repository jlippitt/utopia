use super::Timer;

pub struct Sweep {
    enabled: bool,
    divider_counter: u32,
    divider_period: u32,
    negate: bool,
    shift: u32,
    complement_mode: bool,
    target: u32,
    reload: bool,
    muted: bool,
}

impl Sweep {
    pub fn new(complement_mode: bool) -> Self {
        Self {
            enabled: false,
            divider_counter: 0,
            divider_period: 0,
            negate: false,
            shift: 0,
            complement_mode,
            target: 0,
            reload: false,
            muted: false,
        }
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn set_control(&mut self, value: u8) {
        self.enabled = (value & 0x80) != 0;
        self.divider_period = ((value >> 4) & 7) as u32;
        self.negate = (value & 0x08) != 0;
        self.shift = (value & 7) as u32;
        self.reload = true;
    }

    pub fn update_target_period(&mut self, period: u32) {
        let amount = period >> self.shift;

        self.target = if self.negate {
            period.saturating_sub(amount + self.complement_mode as u32)
        } else {
            period + amount
        };

        self.muted = period < 8 || self.target > 0x07ff;
    }

    pub fn step(&mut self, timer: &mut Timer) {
        if self.divider_counter == 0 && self.enabled && !self.muted {
            timer.set_period(self.target);
            self.update_target_period(self.target);
        }

        if self.divider_counter == 0 || self.reload {
            self.reload = false;
            self.divider_counter = self.divider_period;
        } else {
            self.divider_counter -= 1;
        }
    }
}

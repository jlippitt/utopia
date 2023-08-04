use super::timer::Timer;

pub struct Sweep {
    timer_period: u32,
    divider_counter: u32,
    divider_period: u32,
    negate: bool,
    shift: u8,
    enabled: bool,
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            timer_period: 0,
            divider_counter: 0,
            divider_period: 0,
            negate: false,
            shift: 0,
            enabled: false,
        }
    }

    pub fn reset(&mut self, timer_period: u32) -> bool {
        self.timer_period = timer_period;
        self.divider_counter = self.divider_period;
        self.enabled = self.divider_period != 0 || self.shift != 0;

        if !self.enabled {
            return false;
        }

        self.calculate_target_period() > Timer::MAX_VALUE
    }

    pub fn set_control(&mut self, value: u8) {
        self.divider_period = (value as u32 >> 4) & 7;
        self.negate = (value & 0x08) != 0;
        self.shift = value & 0x07;
    }

    pub fn step(&mut self, timer: &mut Timer) -> bool {
        if !self.enabled || self.divider_counter == 0 {
            return false;
        }

        self.divider_counter -= 1;

        if self.divider_counter != 0 {
            return false;
        }

        self.divider_counter = self.divider_period;

        let target_period = self.calculate_target_period();

        if target_period <= Timer::MAX_VALUE && self.shift != 0 {
            self.timer_period = target_period;
            timer.set_period(target_period);
        }

        self.calculate_target_period() > Timer::MAX_VALUE
    }

    fn calculate_target_period(&mut self) -> u32 {
        let amount = self.timer_period >> self.shift;

        if self.negate {
            self.timer_period.saturating_sub(amount)
        } else {
            self.timer_period + amount
        }
    }
}

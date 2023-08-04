use super::timer::Timer;

pub struct Sweep {
    frequency: u32,
    divider_counter: u32,
    divider_period: u32,
    negate: bool,
    shift: u8,
    enabled: bool,
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            divider_counter: 0,
            divider_period: 0,
            negate: false,
            shift: 0,
            enabled: false,
        }
    }

    pub fn reset(&mut self, timer_period: u32) -> bool {
        self.frequency = timer_period;
        self.divider_counter = self.divider_period;
        self.enabled = self.divider_period != 0 || self.shift != 0;

        if !self.enabled {
            return false;
        }

        self.calculate_target_frequency() > Timer::MAX_FREQUENCY
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

        let target_frequency = self.calculate_target_frequency();

        if target_frequency <= Timer::MAX_FREQUENCY && self.shift != 0 {
            self.frequency = target_frequency;
            timer.set_frequency(target_frequency);
        }

        self.calculate_target_frequency() > Timer::MAX_FREQUENCY
    }

    fn calculate_target_frequency(&mut self) -> u32 {
        let amount = self.frequency >> self.shift;

        if self.negate {
            self.frequency.saturating_sub(amount)
        } else {
            self.frequency + amount
        }
    }
}

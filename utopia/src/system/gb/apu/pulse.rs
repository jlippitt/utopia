use super::component::{LengthCounter, Timer};
use crate::util::audio::Sequencer;

const DUTY_CYCLE: [[u8; 8]; 4] = [
    [1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 1],
];

pub struct Pulse {
    timer: Timer,
    sequencer: Sequencer<8>,
    length_counter: LengthCounter,
    enabled: bool,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            length_counter: LengthCounter::new(),
            sequencer: Sequencer::new(&DUTY_CYCLE[0]),
            enabled: false,
        }
    }

    pub fn output(&self) -> u8 {
        if self.enabled {
            // TODO: Envelope
            self.sequencer.output() * 15
        } else {
            0
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0 => {
                // TODO: Sweep
            }
            1 => {
                let duty_cycle = value as usize >> 6;
                self.sequencer.set_sequence(&DUTY_CYCLE[duty_cycle]);
                self.length_counter.set_period(value as u32 & 0x3f);
            }
            2 => {
                // TODO: Envelope
            }
            3 => self.timer.set_period_low(value),
            4 => {
                self.timer.set_period_high(value & 0x07);
                self.length_counter.set_enabled((value & 0x40) != 0);

                if (value & 0x80) != 0 {
                    self.enabled = true;
                    self.length_counter.reset();
                    self.timer.reset();
                    // TODO: Envelope
                    // TODO: Sweep
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self) {
        if self.timer.step() {
            self.sequencer.step();
        }
    }

    pub fn on_divider_clock(&mut self, divider: u64) {
        if (divider & 1) == 0 && self.length_counter.step() {
            self.enabled = false;
        }
    }
}

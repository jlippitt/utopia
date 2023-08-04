use super::component::{Envelope, LengthCounter, Sweep, Timer};
use crate::util::audio::Sequencer;

const DUTY_CYCLE: [[u8; 8]; 4] = [
    [1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 1],
];

pub struct Pulse {
    enabled: bool,
    timer: Timer,
    sequencer: Sequencer<8>,
    length_counter: LengthCounter,
    sweep: Option<Sweep>,
    envelope: Envelope,
}

impl Pulse {
    pub fn new(sweep_enabled: bool) -> Self {
        Self {
            enabled: false,
            timer: Timer::new(Timer::MAX_PERIOD),
            sequencer: Sequencer::new(&DUTY_CYCLE[0]),
            length_counter: LengthCounter::new(63),
            sweep: sweep_enabled.then(|| Sweep::new()),
            envelope: Envelope::new(),
        }
    }

    pub fn output(&self) -> u8 {
        if self.enabled {
            self.sequencer.output() * self.envelope.volume()
        } else {
            0
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0 => {
                if let Some(sweep) = &mut self.sweep {
                    sweep.set_control(value);
                }
            }
            1 => {
                let duty_cycle = value as usize >> 6;
                self.sequencer.set_sequence(&DUTY_CYCLE[duty_cycle]);
                self.length_counter.set_period(value as u32 & 0x3f);
            }
            2 => self.envelope.set_control(value),
            3 => self.timer.set_frequency_low(value),
            4 => {
                self.timer.set_frequency_high(value & 0x07);
                self.length_counter.set_enabled((value & 0x40) != 0);

                if (value & 0x80) != 0 {
                    self.enabled = true;
                    self.timer.reset();
                    self.length_counter.reset();
                    self.envelope.reset();

                    if let Some(sweep) = &mut self.sweep {
                        if sweep.reset(self.timer.frequency()) {
                            self.enabled = false;
                        }
                    }
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

        if (divider & 3) == 2 {
            if let Some(sweep) = &mut self.sweep {
                if sweep.step(&mut self.timer) {
                    self.enabled = false;
                }
            }
        }

        if (divider & 7) == 7 {
            self.envelope.step();
        }
    }
}

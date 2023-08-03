use super::component::Timer;
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
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            sequencer: Sequencer::new(&DUTY_CYCLE[0]),
        }
    }

    pub fn output(&self) -> u8 {
        // TODO: Envelope
        self.sequencer.output() * 15
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0 => {
                // TODO: Sweep
            }
            1 => {
                let duty_cycle = value as usize >> 6;
                self.sequencer.set_sequence(&DUTY_CYCLE[duty_cycle]);
                // TODO: Length counter
            }
            2 => {
                // TODO: Envelope
            }
            3 => self.timer.set_period_low(value),
            4 => {
                self.timer.set_period_high(value & 0x07);
                // TODO: Length counter
                // TODO: Trigger
            }
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self) {
        if self.timer.step() {
            self.sequencer.step();
        }
    }
}

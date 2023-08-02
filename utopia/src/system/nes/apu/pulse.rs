use super::component::{Sequencer, Timer};

const DUTY_CYCLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Pulse {
    timer: Timer,
    sequencer: Sequencer<8>,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(1),
            sequencer: Sequencer::new(&DUTY_CYCLE[0]),
        }
    }

    pub fn sample(&self) -> u8 {
        self.sequencer.sample() * 15
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => {
                let duty_cycle = ((value >> 6) & 3) as usize;
                self.sequencer.set_sequence(&DUTY_CYCLE[duty_cycle]);

                // TODO: Length counter
                // TODO: Envelope
            }
            1 => {
                // TODO: Sweep
            }
            2 => self.timer.set_period_low(value),
            3 => {
                self.timer.set_period_high(value & 0x07);
                // TODO: Length counter

                self.sequencer.reset();
                // TODO: Restart envelope
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

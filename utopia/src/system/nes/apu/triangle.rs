use super::component::{LengthCounter, LinearCounter, Sequencer, Timer};
use super::frame::FrameEvent;

#[rustfmt::skip]
const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

pub struct Triangle {
    timer: Timer,
    sequencer: Sequencer<32>,
    linear_counter: LinearCounter,
    length_counter: LengthCounter,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(0, 0),
            sequencer: Sequencer::new(&SEQUENCE),
            linear_counter: LinearCounter::new(),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.length_counter.counter() != 0
    }

    pub fn output(&self) -> u8 {
        if self.timer.period() >= 2 {
            self.sequencer.output()
        } else {
            // Silenced to avoid popping sound
            // This should technically be a value of 7.5
            7
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => {
                self.length_counter.set_halted((value & 0x80) != 0);
                self.linear_counter.set_control(value);
            }
            1 => (),
            2 => self.timer.set_period_low(value),
            3 => {
                self.timer.set_period_high(value & 0x07);
                self.length_counter.load(value >> 3);
                self.linear_counter.reset();
            }
            _ => unreachable!(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    pub fn step(&mut self) {
        if self.timer.step()
            && self.linear_counter.counter() != 0
            && self.length_counter.counter() != 0
        {
            self.sequencer.step();
        }
    }

    pub fn on_frame_event(&mut self, event: FrameEvent) {
        self.linear_counter.step();

        if event == FrameEvent::Half {
            self.length_counter.step();
        }
    }
}

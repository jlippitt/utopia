use super::component::{Envelope, LengthCounter, Sequencer, Sweep, Timer};
use super::frame::FrameEvent;

const DUTY_CYCLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Pulse {
    timer: Timer,
    sequencer: Sequencer<8>,
    envelope: Envelope,
    sweep: Sweep,
    length_counter: LengthCounter,
}

impl Pulse {
    pub fn new(complement_mode: bool) -> Self {
        Self {
            timer: Timer::new(1),
            sequencer: Sequencer::new(&DUTY_CYCLE[0]),
            envelope: Envelope::new(),
            sweep: Sweep::new(complement_mode),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.length_counter.counter() != 0
    }

    pub fn sample(&self) -> u8 {
        if !self.sweep.muted() && self.length_counter.counter() != 0 {
            self.sequencer.sample() * self.envelope.volume()
        } else {
            0
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => {
                let duty_cycle = ((value >> 6) & 3) as usize;
                self.sequencer.set_sequence(&DUTY_CYCLE[duty_cycle]);
                self.length_counter.set_halted((value & 0x20) != 0);
                self.envelope.set_control(value);
            }
            1 => self.sweep.set_control(value),
            2 => {
                self.timer.set_period_low(value);
                self.sweep.update_target_period(self.timer.period());
            }
            3 => {
                self.timer.set_period_high(value & 0x07);
                self.sweep.update_target_period(self.timer.period());
                self.length_counter.load(value >> 3);
                self.sequencer.reset();
                self.envelope.reset();
            }
            _ => unreachable!(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    pub fn step(&mut self) {
        if self.timer.step() {
            self.sequencer.step();
        }
    }

    pub fn on_frame_event(&mut self, event: FrameEvent) {
        self.envelope.step();

        if event == FrameEvent::Half {
            self.sweep.step(&mut self.timer);
            self.length_counter.step();
        }
    }
}

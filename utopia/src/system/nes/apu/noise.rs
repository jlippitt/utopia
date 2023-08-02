use super::component::{Envelope, LengthCounter, Timer};
use super::frame::FrameEvent;

#[rustfmt::skip]
const PERIODS: [u32; 16] = [
       4,    8,   16,   32,   64,   96,  128,  160,
     202,  254,  380,  508,  762, 1016, 2034, 4068,
];

pub struct Noise {
    timer: Timer,
    envelope: Envelope,
    length_counter: LengthCounter,
    mode: bool,
    shift: u16,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            shift: 1,
            timer: Timer::new(1),
            mode: false,
            envelope: Envelope::new(),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.length_counter.counter() != 0
    }

    pub fn sample(&self) -> u8 {
        if (self.shift & 1) == 0 && self.length_counter.counter() != 0 {
            self.envelope.volume()
        } else {
            0
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => {
                self.length_counter.set_halted((value & 0x20) != 0);
                self.envelope.set_control(value);
            }
            1 => (),
            2 => {
                self.mode = (value & 0x80) != 0;
                self.timer.set_period(PERIODS[(value & 0x0f) as usize]);
            }
            3 => {
                self.length_counter.load(value >> 3);
                self.envelope.reset();
            }
            _ => unreachable!(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    pub fn step(&mut self) {
        if !self.timer.step() {
            return;
        }

        let xor_bit = if self.mode { 6 } else { 1 };
        let feedback = (self.shift ^ (self.shift >> xor_bit)) & 1;
        self.shift = (feedback << 14) | (self.shift >> 1);
    }

    pub fn on_frame_event(&mut self, event: FrameEvent) {
        self.envelope.step();

        if event == FrameEvent::Half {
            // TODO: Sweep
            self.length_counter.step();
        }
    }
}

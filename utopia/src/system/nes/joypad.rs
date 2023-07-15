use crate::JoypadState;
use tracing::debug;

pub struct Joypad {
    current_state: [u8; 2],
    polled_state: [u8; 2],
    latch: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            current_state: [0; 2],
            polled_state: [0xff; 2],
            latch: false,
        }
    }

    pub fn update(&mut self, joypad_state: &JoypadState) {
        self.current_state[0] = 0;

        if joypad_state.a {
            self.current_state[0] |= 0x01;
        }

        if joypad_state.b {
            self.current_state[0] |= 0x02;
        }

        if joypad_state.select {
            self.current_state[0] |= 0x04;
        }

        if joypad_state.start {
            self.current_state[0] |= 0x08;
        }

        if joypad_state.up {
            self.current_state[0] |= 0x10;
        }

        if joypad_state.down {
            self.current_state[0] |= 0x20;
        }

        if joypad_state.left {
            self.current_state[0] |= 0x40;
        }

        if joypad_state.right {
            self.current_state[0] |= 0x80;
        }
    }

    pub fn read_register(&mut self, address: u16, prev_value: u8) -> u8 {
        let index = (address & 1) as usize;

        let value = if self.latch {
            self.current_state[index] & 0x01
        } else {
            let value = self.polled_state[index] & 0x01;
            self.polled_state[index] = 0x80 | (self.polled_state[index] >> 1);
            value
        };

        (prev_value & 0xf8) | value
    }

    pub fn write_register(&mut self, value: u8) {
        let latch = (value & 0x01) != 0;

        if self.latch && !latch {
            self.polled_state = self.current_state;
            debug!("Joypad State Latched");
        }

        self.latch = latch;
    }
}

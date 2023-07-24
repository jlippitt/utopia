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

    pub fn update(&mut self, new_state: &JoypadState) {
        self.current_state[0] = 0;
        self.current_state[0] |= if new_state.a { 0x01 } else { 0 };
        self.current_state[0] |= if new_state.b { 0x02 } else { 0 };
        self.current_state[0] |= if new_state.select { 0x04 } else { 0 };
        self.current_state[0] |= if new_state.start { 0x08 } else { 0 };
        self.current_state[0] |= if new_state.up { 0x10 } else { 0 };
        self.current_state[0] |= if new_state.down { 0x20 } else { 0 };
        self.current_state[0] |= if new_state.left { 0x40 } else { 0 };
        self.current_state[0] |= if new_state.right { 0x80 } else { 0 };
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

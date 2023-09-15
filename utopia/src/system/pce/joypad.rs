use crate::JoypadState;

pub struct Joypad {
    select: bool,
    clear: bool,
    actions: u8,
    directions: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            select: false,
            clear: false,
            actions: 0,
            directions: 0,
        }
    }

    pub fn update(&mut self, state: &JoypadState) {
        self.actions = 0;
        self.actions |= if !state.buttons[1] { 0x01 } else { 0 };
        self.actions |= if !state.buttons[0] { 0x02 } else { 0 };
        self.actions |= if !state.buttons[8] { 0x04 } else { 0 };
        self.actions |= if !state.buttons[9] { 0x08 } else { 0 };

        self.directions = 0;
        self.directions |= if !state.buttons[12] { 0x01 } else { 0 };
        self.directions |= if !state.buttons[15] { 0x02 } else { 0 };
        self.directions |= if !state.buttons[13] { 0x04 } else { 0 };
        self.directions |= if !state.buttons[14] { 0x08 } else { 0 };
    }

    pub fn read(&self) -> u8 {
        if self.clear {
            0
        } else if self.select {
            self.directions
        } else {
            self.actions
        }
    }

    pub fn write(&mut self, value: u8) {
        self.select = (value & 0x01) != 0;
        self.clear = (value & 0x02) != 0;
    }
}

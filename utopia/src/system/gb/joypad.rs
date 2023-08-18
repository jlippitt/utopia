use crate::JoypadState;
use bitflags::bitflags;
use tracing::debug;

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct Select: u8 {
        const DIRECTION = 0x10;
        const ACTION = 0x20;
    }
}

pub struct Joypad {
    action_state: u8,
    direction_state: u8,
    select: Select,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            action_state: 0,
            direction_state: 0,
            select: Select::empty(),
        }
    }

    pub fn update(&mut self, state: &JoypadState) {
        let JoypadState { buttons, .. } = &state;

        self.direction_state = 0;
        self.direction_state |= if buttons[15] { 0x01 } else { 0 };
        self.direction_state |= if buttons[14] { 0x02 } else { 0 };
        self.direction_state |= if buttons[12] { 0x04 } else { 0 };
        self.direction_state |= if buttons[13] { 0x08 } else { 0 };

        self.action_state = 0;
        self.action_state |= if buttons[1] { 0x01 } else { 0 };
        self.action_state |= if buttons[0] { 0x02 } else { 0 };
        self.action_state |= if buttons[8] { 0x04 } else { 0 };
        self.action_state |= if buttons[9] { 0x08 } else { 0 };
    }

    pub fn read(&self) -> u8 {
        let mut value = 0xff;

        if self.select.contains(Select::DIRECTION) {
            value &= !self.direction_state;
        }

        if self.select.contains(Select::ACTION) {
            value &= !self.action_state;
        }

        value
    }

    pub fn write(&mut self, value: u8) {
        self.select = Select::from_bits_truncate(!value);
        debug!("Joypad Select: {:?}", self.select);
    }
}

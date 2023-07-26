use crate::JoypadState;
use tracing::{debug, warn};

pub struct Joypad {
    current_state: [u16; 4],
    polled_state: [u16; 4],
    auto_read_state: [u16; 4],
    auto_read_enabled: bool,
    latch: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            current_state: [0; 4],
            polled_state: [0; 4],
            auto_read_state: [0; 4],
            auto_read_enabled: false,
            latch: false,
        }
    }

    pub fn auto_read_state_low(&self, index: usize) -> u8 {
        self.auto_read_state[index] as u8
    }

    pub fn auto_read_state_high(&self, index: usize) -> u8 {
        (self.auto_read_state[index] >> 8) as u8
    }

    pub fn update(&mut self, new_state: &JoypadState) {
        self.current_state[0] = 0;
        self.current_state[0] |= if new_state.b { 0x8000 } else { 0 };
        self.current_state[0] |= if new_state.y { 0x4000 } else { 0 };
        self.current_state[0] |= if new_state.select { 0x2000 } else { 0 };
        self.current_state[0] |= if new_state.start { 0x1000 } else { 0 };
        self.current_state[0] |= if new_state.up { 0x0800 } else { 0 };
        self.current_state[0] |= if new_state.down { 0x0400 } else { 0 };
        self.current_state[0] |= if new_state.left { 0x0200 } else { 0 };
        self.current_state[0] |= if new_state.right { 0x0100 } else { 0 };
        self.current_state[0] |= if new_state.a { 0x0080 } else { 0 };
        self.current_state[0] |= if new_state.x { 0x0040 } else { 0 };
        self.current_state[0] |= if new_state.l { 0x0020 } else { 0 };
        self.current_state[0] |= if new_state.r { 0x0010 } else { 0 };
    }

    pub fn read_serial(&mut self, address: u8, prev_value: u8) -> u8 {
        match address {
            0x16 => {
                let mut value = prev_value & 0xfc;
                value |= if self.read_bit(0) { 0x01 } else { 0 };
                value |= if self.read_bit(2) { 0x02 } else { 0 };
                value
            }
            0x17 => {
                let mut value = (prev_value & 0xe0) | 0x1c;
                value |= if self.read_bit(1) { 0x01 } else { 0 };
                value |= if self.read_bit(3) { 0x02 } else { 0 };
                value
            }
            _ => {
                warn!("Unmapped serial joypad read: {:02X}", address);
                prev_value
            }
        }
    }

    pub fn write_serial(&mut self, address: u8, value: u8) {
        if address != 0x16 {
            warn!(
                "Unmapped serial joypad write: {:02X} <= {:02X}",
                address, value
            );
            return;
        }

        let latch = (value & 0x01) != 0;

        if self.latch && !latch {
            self.polled_state = self.current_state;
            debug!("Joypad State Latched: {:04X}", self.polled_state[0]);
        }

        self.latch = latch;
    }

    pub fn set_auto_read_enabled(&mut self, enabled: bool) {
        self.auto_read_enabled = enabled;
        debug!("Joypad Auto-Read Enabled: {}", self.auto_read_enabled);
    }

    pub fn perform_auto_read(&mut self) {
        if !self.auto_read_enabled {
            return;
        }

        // TODO: Actual timing for this
        self.auto_read_state = self.current_state;
        self.polled_state = [0xffff, 0xffff, 0xffff, 0xffff];
        self.latch = false;
        debug!("Joypad Auto-Read Complete");
    }

    fn read_bit(&mut self, index: usize) -> bool {
        if self.latch {
            (self.current_state[index] & 0x8000) != 0
        } else {
            let result = (self.polled_state[index] & 0x8000) != 0;
            self.polled_state[index] = (self.polled_state[index] << 1) | 0x0001;
            result
        }
    }
}

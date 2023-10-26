use super::WIDTH;
use tracing::trace;

pub type BoolMask = [bool; WIDTH / 2];

pub const MASK_NONE: BoolMask = [false; WIDTH / 2];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Operator {
    Or,
    And,
    Xor,
    Xnor,
}

pub struct Window {
    left: u8,
    right: u8,
    name: &'static str,
}

pub struct WindowMask {
    dirty: bool,
    w1_enabled: bool,
    w1_inverted: bool,
    w2_enabled: bool,
    w2_inverted: bool,
    operator: Operator,
    name: &'static str,
    mask: BoolMask,
}

impl Window {
    pub fn new(name: &'static str) -> Self {
        Self {
            left: 0,
            right: 0,
            name,
        }
    }

    pub fn set_left(&mut self, value: u8) -> bool {
        let modified = value != self.left;
        self.left = value;
        trace!("{} Left: {}", self.name, self.left);
        modified
    }

    pub fn set_right(&mut self, value: u8) -> bool {
        let modified = value != self.right;
        self.right = value;
        trace!("{} Right: {}", self.name, self.right);
        modified
    }
}

impl WindowMask {
    pub fn new(name: &'static str) -> Self {
        Self {
            dirty: true,
            w1_enabled: false,
            w1_inverted: false,
            w2_enabled: false,
            w2_inverted: false,
            operator: Operator::Or,
            name,
            mask: MASK_NONE,
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.w1_inverted = (value & 0x01) != 0;
        self.w1_enabled = (value & 0x02) != 0;
        self.w2_inverted = (value & 0x04) != 0;
        self.w2_enabled = (value & 0x08) != 0;
        trace!("{} W1 Enabled: {}", self.name, self.w1_enabled);
        trace!("{} W1 Inverted: {}", self.name, self.w1_inverted);
        trace!("{} W2 Enabled: {}", self.name, self.w2_enabled);
        trace!("{} W2 Inverted: {}", self.name, self.w2_inverted);
        self.dirty = true;
    }

    pub fn set_operator(&mut self, value: u8) {
        self.operator = match value {
            0 => Operator::Or,
            1 => Operator::And,
            2 => Operator::Xor,
            3 => Operator::Xnor,
            _ => panic!("Invalid window mask operator: {}", value),
        };
        trace!("{} Operator: {:?}", self.name, self.operator);
        self.dirty = true;
    }

    pub fn mark_as_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mask(&mut self, windows: &[Window; 2]) -> &BoolMask {
        if self.dirty {
            if self.w1_enabled {
                self.build_mask(&windows[0], self.w1_inverted);

                if self.w2_enabled {
                    self.apply_operator(&windows[1]);
                }
            } else if self.w2_enabled {
                self.build_mask(&windows[1], self.w2_inverted)
            } else {
                self.mask.fill(false);
            }

            trace!("{} Updated", self.name);
        }

        &self.mask
    }

    fn build_mask(&mut self, window: &Window, inverted: bool) {
        let left = window.left as usize;
        let right = window.right as usize;

        if left <= right {
            self.mask[0..left].fill(inverted);
            self.mask[left..=right].fill(!inverted);
            self.mask[(right + 1)..].fill(inverted);
        } else {
            self.mask.fill(inverted);
        }
    }

    fn apply_operator(&mut self, w2: &Window) {
        let left = w2.left as usize;
        let right = w2.right as usize;
        let mut index = 0;

        while index < left {
            self.mask[index] = self.operator.apply(self.mask[index], self.w2_inverted);
            index += 1;
        }

        while index <= right {
            self.mask[index] = self.operator.apply(self.mask[index], !self.w2_inverted);
            index += 1;
        }

        while index < self.mask.len() {
            self.mask[index] = self.operator.apply(self.mask[index], self.w2_inverted);
            index += 1;
        }
    }
}

impl Operator {
    fn apply(self, lhs: bool, rhs: bool) -> bool {
        match self {
            Operator::Or => lhs || rhs,
            Operator::And => lhs && rhs,
            Operator::Xor => lhs != rhs,
            Operator::Xnor => lhs == rhs,
        }
    }
}

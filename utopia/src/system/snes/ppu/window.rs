use tracing::debug;

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
    w1_enabled: bool,
    w1_inverted: bool,
    w2_enabled: bool,
    w2_inverted: bool,
    operator: Operator,
    name: &'static str,
}

impl Window {
    pub fn new(name: &'static str) -> Self {
        Self {
            left: 0,
            right: 0,
            name,
        }
    }

    pub fn set_left(&mut self, value: u8) {
        self.left = value;
        debug!("{} Left: {}", self.name, self.left);
    }

    pub fn set_right(&mut self, value: u8) {
        self.right = value;
        debug!("{} Right: {}", self.name, self.right);
    }
}

impl WindowMask {
    pub fn new(name: &'static str) -> Self {
        Self {
            w1_enabled: false,
            w1_inverted: false,
            w2_enabled: false,
            w2_inverted: false,
            operator: Operator::Or,
            name,
        }
    }

    pub fn set_control(&mut self, value: u8) {
        self.w1_inverted = (value & 0x01) != 0;
        self.w1_enabled = (value & 0x02) != 0;
        self.w2_inverted = (value & 0x04) != 0;
        self.w2_enabled = (value & 0x08) != 0;
        debug!("{} Mask W1 Enabled: {}", self.name, self.w1_enabled);
        debug!("{} Mask W1 Inverted: {}", self.name, self.w1_inverted);
        debug!("{} Mask W2 Enabled: {}", self.name, self.w2_enabled);
        debug!("{} Mask W2 Inverted: {}", self.name, self.w2_inverted);
    }

    pub fn set_operator(&mut self, value: u8) {
        self.operator = match value {
            0 => Operator::Or,
            1 => Operator::And,
            2 => Operator::Xor,
            3 => Operator::Xnor,
            _ => panic!("Invalid window mask operator: {}", value),
        };
        debug!("{} Mask Operator: {:?}", self.name, self.operator);
    }
}

use tracing::debug;

pub struct Window {
    left: u8,
    right: u8,
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

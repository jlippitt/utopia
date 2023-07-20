use tracing::debug;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScreenToggle {
    Main = 0x01,
    Sub = 0x02,
}

#[derive(Copy, Clone)]
pub struct Toggle {
    enabled: u8,
    name: &'static str,
}

impl Toggle {
    pub fn new(name: &'static str) -> Self {
        Self { enabled: 0, name }
    }

    pub fn any_enabled(&self) -> bool {
        self.enabled != 0
    }

    pub fn screen_enabled(&self, screen: ScreenToggle) -> bool {
        self.enabled & (screen as u8) != 0
    }

    pub fn set(&mut self, screen: ScreenToggle, enabled: bool) {
        if enabled {
            self.enabled |= screen as u8;
        } else {
            self.enabled &= !(screen as u8);
        }

        debug!("{} {:?} Screen Enabled: {}", self.name, screen, enabled);
    }
}

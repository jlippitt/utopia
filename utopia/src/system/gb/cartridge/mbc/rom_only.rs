use super::Mbc;

pub struct RomOnly;

impl RomOnly {
    pub fn new() -> Self {
        Self
    }
}

impl Mbc for RomOnly {
    // Default behaviour
}

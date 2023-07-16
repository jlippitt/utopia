use crate::core::spc700::{Bus, Core};
use std::fmt;

pub struct Apu {
    core: Core<Hardware>,
}

impl Apu {
    pub fn new() -> Self {
        let hw = Hardware::new();
        let core = Core::new(hw);
        println!("{}", core);
        Self { core }
    }

    pub fn read(&self, _address: u8) -> u8 {
        todo!("APU reads");
    }

    pub fn write(&self, _address: u8, _value: u8) {
        todo!("APU writes");
    }
}

struct Hardware {
    //
}

impl Hardware {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bus for Hardware {
    //
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

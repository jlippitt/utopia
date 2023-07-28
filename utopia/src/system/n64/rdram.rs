use crate::util::facade::DataReader;

pub struct Interface {
    //
}

pub struct Rdram {
    interface: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            interface: Interface {},
        }
    }

    pub fn interface(&self) -> &Interface {
        &self.interface
    }
}

impl DataReader for Interface {
    type Address = u32;
    type Value = u32;

    fn read(&self, _address: u32) -> u32 {
        // TODO
        0
    }
}

pub struct Apu {
    //
}

impl Apu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, _address: u8) -> u8 {
        todo!("APU reads");
    }

    pub fn write(&self, _address: u8, _value: u8) {
        todo!("APU writes");
    }
}

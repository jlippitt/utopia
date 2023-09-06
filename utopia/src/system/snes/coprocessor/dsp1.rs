use super::Coprocessor;

pub struct Dsp1 {
    //
}

impl Dsp1 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Coprocessor for Dsp1 {
    fn read(&mut self, page_type: u32, address: u32, _prev_value: u8) -> u8 {
        if ((address >> page_type) & 1) == 0 {
            todo!("Data Register Read");
        } else {
            todo!("Status Register Read");
        }
    }

    fn write(&mut self, page_type: u32, address: u32, _value: u8) {
        if ((address >> page_type) & 1) == 0 {
            todo!("Data Register Write");
        } else {
            todo!("Status Register Write");
        }
    }

    fn step(&mut self) {
        //
    }
}

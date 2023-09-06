use dsp1::Dsp1;
use enum_dispatch::enum_dispatch;

mod dsp1;

#[enum_dispatch]
pub trait Coprocessor {
    fn read(&mut self, page_type: u32, address: u32, prev_value: u8) -> u8;
    fn write(&mut self, page_type: u32, address: u32, value: u8);
    fn step(&mut self);
}

#[enum_dispatch(Coprocessor)]
pub enum CoprocessorType {
    Dsp1,
}

pub fn create_coprocessor(cartridge_type: u8) -> Option<CoprocessorType> {
    if (cartridge_type & 0x0f) < 0x03 {
        return None;
    }

    Some(match cartridge_type & 0xf0 {
        0x00 => CoprocessorType::Dsp1(Dsp1::new()),
        value => unimplemented!("Coprocessor Type: {:02X}", value),
    })
}

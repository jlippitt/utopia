use super::super::{Bus, Core};

pub trait WriteOperator {
    const NAME: &'static str;
    fn apply8(core: &Core<impl Bus>) -> u8;
    fn apply16(core: &Core<impl Bus>) -> u16;
}

pub struct Stz;

impl WriteOperator for Stz {
    const NAME: &'static str = "STZ";

    fn apply8(_core: &Core<impl Bus>) -> u8 {
        0
    }

    fn apply16(_core: &Core<impl Bus>) -> u16 {
        0
    }
}

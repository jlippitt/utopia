use super::super::{Bus, Core};

pub trait WriteOperator {
    const NAME: &'static str;
    fn apply(core: &Core<impl Bus>) -> u8;
}

pub struct Sta;

impl WriteOperator for Sta {
    const NAME: &'static str = "STA";

    fn apply(core: &Core<impl Bus>) -> u8 {
        core.a
    }
}

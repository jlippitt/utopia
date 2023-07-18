use super::super::{Bus, Core};

pub trait UnaryOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8;
}

pub struct Dec;

impl UnaryOperator for Dec {
    const NAME: &'static str = "DEC";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        core.set_nz(result);
        result
    }
}

pub struct Inc;

impl UnaryOperator for Inc {
    const NAME: &'static str = "INC";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        core.set_nz(result);
        result
    }
}

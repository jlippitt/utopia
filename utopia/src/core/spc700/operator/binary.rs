use super::super::{Bus, Core};

pub trait BinaryOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8;
}

pub struct Mov;

impl BinaryOperator for Mov {
    const NAME: &'static str = "MOV";

    fn apply(core: &mut Core<impl Bus>, _lhs: u8, rhs: u8) -> u8 {
        core.set_nz(rhs);
        rhs
    }
}

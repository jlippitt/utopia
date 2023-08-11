use super::super::{Bus, Core};

pub trait MultiplyOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32, acc: u32)
        -> u32;
}

pub struct Mul;

impl MultiplyOperator for Mul {
    const NAME: &'static str = "MUL";

    fn apply<const SET_FLAGS: bool>(
        core: &mut Core<impl Bus>,
        lhs: u32,
        rhs: u32,
        _acc: u32,
    ) -> u32 {
        let result = lhs * rhs;

        if SET_FLAGS {
            core.set_nz(result);
            // TODO: Carry is set to meaningless value?
        }

        result
    }
}

use super::{Bus, Core};

pub trait BinaryOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32;
}

pub trait MoveOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, value: u32) -> u32;
}

pub trait ComparisonOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32);
}

pub struct Add;

impl BinaryOperator for Add {
    const NAME: &'static str = "ADD";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(lhs, rhs, false)
    }
}

pub struct Adc;

impl BinaryOperator for Adc {
    const NAME: &'static str = "ADC";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(lhs, rhs, true)
    }
}

pub struct Tst;

impl ComparisonOperator for Tst {
    const NAME: &'static str = "TST";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.set_nz(lhs & rhs);
    }
}

pub struct Teq;

impl ComparisonOperator for Teq {
    const NAME: &'static str = "TEQ";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.set_nz(lhs ^ rhs);
    }
}

pub struct Cmp;

impl ComparisonOperator for Cmp {
    const NAME: &'static str = "CMP";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, !rhs, true);
    }
}

pub struct Cmn;

impl ComparisonOperator for Cmn {
    const NAME: &'static str = "CMN";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, rhs, false);
    }
}

pub struct Mov;

impl MoveOperator for Mov {
    const NAME: &'static str = "MOV";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, value: u32) -> u32 {
        if SET_FLAGS {
            core.set_nz(value);
        }

        value
    }
}

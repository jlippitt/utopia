use super::super::{Bus, Core};

pub trait BinaryOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32;
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

pub struct And;

impl BinaryOperator for And {
    const NAME: &'static str = "AND";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        let result = lhs & rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

pub struct Eor;

impl BinaryOperator for Eor {
    const NAME: &'static str = "EOR";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        let result = lhs ^ rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

pub struct Orr;

impl BinaryOperator for Orr {
    const NAME: &'static str = "ORR";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        let result = lhs | rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

pub struct Bic;

impl BinaryOperator for Bic {
    const NAME: &'static str = "BIC";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        let result = lhs & !rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

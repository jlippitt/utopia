use super::super::{Bus, Core};

pub trait BinaryOperator {
    const NAME: &'static str;
    const LOGICAL: bool;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32;
}

pub struct Add;

impl BinaryOperator for Add {
    const NAME: &'static str = "ADD";
    const LOGICAL: bool = false;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(lhs, rhs, false)
    }
}

pub struct Adc;

impl BinaryOperator for Adc {
    const NAME: &'static str = "ADC";
    const LOGICAL: bool = false;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(lhs, rhs, core.cpsr.c)
    }
}

pub struct Sub;

impl BinaryOperator for Sub {
    const NAME: &'static str = "SUB";
    const LOGICAL: bool = false;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(lhs, !rhs, true)
    }
}

pub struct Sbc;

impl BinaryOperator for Sbc {
    const NAME: &'static str = "SBC";
    const LOGICAL: bool = false;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(lhs, !rhs, core.cpsr.c)
    }
}

pub struct Rsb;

impl BinaryOperator for Rsb {
    const NAME: &'static str = "RSB";
    const LOGICAL: bool = false;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(rhs, !lhs, true)
    }
}

pub struct Rsc;

impl BinaryOperator for Rsc {
    const NAME: &'static str = "RSC";
    const LOGICAL: bool = false;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        core.add_with_carry::<SET_FLAGS>(rhs, !lhs, core.cpsr.c)
    }
}

pub struct And;

impl BinaryOperator for And {
    const NAME: &'static str = "AND";
    const LOGICAL: bool = true;

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
    const LOGICAL: bool = true;

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
    const LOGICAL: bool = true;

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
    const LOGICAL: bool = true;

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32 {
        let result = lhs & !rhs;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

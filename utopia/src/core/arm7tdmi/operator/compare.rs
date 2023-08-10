use super::super::{Bus, Core};

pub trait CompareOperator {
    const NAME: &'static str;
    const LOGICAL: bool;
    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32);
}

pub struct Tst;

impl CompareOperator for Tst {
    const NAME: &'static str = "TST";
    const LOGICAL: bool = true;

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.set_nz(lhs & rhs);
    }
}

pub struct Teq;

impl CompareOperator for Teq {
    const NAME: &'static str = "TEQ";
    const LOGICAL: bool = true;

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.set_nz(lhs ^ rhs);
    }
}

pub struct Cmp;

impl CompareOperator for Cmp {
    const NAME: &'static str = "CMP";
    const LOGICAL: bool = false;

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, !rhs, true);
    }
}

pub struct Cmn;

impl CompareOperator for Cmn {
    const NAME: &'static str = "CMN";
    const LOGICAL: bool = false;

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, rhs, false);
    }
}

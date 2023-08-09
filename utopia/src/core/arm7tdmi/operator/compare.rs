use super::super::{Bus, Core};

pub trait CompareOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32);
}

pub struct Tst;

impl CompareOperator for Tst {
    const NAME: &'static str = "TST";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.set_nz(lhs & rhs);
    }
}

pub struct Teq;

impl CompareOperator for Teq {
    const NAME: &'static str = "TEQ";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.set_nz(lhs ^ rhs);
    }
}

pub struct Cmp;

impl CompareOperator for Cmp {
    const NAME: &'static str = "CMP";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, !rhs, true);
    }
}

pub struct Cmn;

impl CompareOperator for Cmn {
    const NAME: &'static str = "CMN";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry::<true>(lhs, rhs, false);
    }
}

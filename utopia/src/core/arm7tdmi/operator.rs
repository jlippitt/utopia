use super::{Bus, Core};

pub trait ComparisonOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32);
}

pub struct Cmp;

impl ComparisonOperator for Cmp {
    const NAME: &'static str = "CMP";

    fn apply(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) {
        core.add_with_carry(lhs, !rhs, true);
    }
}

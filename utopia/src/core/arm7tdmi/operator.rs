use super::{Bus, Core};

// pub trait BinaryOperator {
//     const NAME: &'static str;
//     fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, lhs: u32, rhs: u32) -> u32;
// }

pub trait MoveOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, value: u32) -> u32;
}

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

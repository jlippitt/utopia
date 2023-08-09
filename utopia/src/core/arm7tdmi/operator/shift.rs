use super::super::{Bus, Core};

pub trait ShiftOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(
        core: &mut Core<impl Bus>,
        value: u32,
        shift_amount: u32,
    ) -> u32;
}

pub struct Lsl;

impl ShiftOperator for Lsl {
    const NAME: &'static str = "LSL";

    fn apply<const SET_FLAGS: bool>(
        core: &mut Core<impl Bus>,
        value: u32,
        shift_amount: u32,
    ) -> u32 {
        let result = value << shift_amount;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

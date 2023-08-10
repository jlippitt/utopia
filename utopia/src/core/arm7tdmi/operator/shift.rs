use super::super::{Bus, Core};

pub trait ShiftOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool, const LOGICAL: bool>(
        core: &mut Core<impl Bus>,
        value: u32,
        shift_amount: u32,
    ) -> u32;
}

pub struct Lsl;

impl ShiftOperator for Lsl {
    const NAME: &'static str = "LSL";

    fn apply<const SET_FLAGS: bool, const LOGICAL: bool>(
        core: &mut Core<impl Bus>,
        value: u32,
        shift_amount: u32,
    ) -> u32 {
        if shift_amount == 0 {
            return value;
        }

        if SET_FLAGS && LOGICAL {
            core.cpsr.c = (value & 1u32.rotate_right(shift_amount)) != 0;
        }

        let result = value << shift_amount;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

pub struct Lsr;

impl ShiftOperator for Lsr {
    const NAME: &'static str = "LSR";

    fn apply<const SET_FLAGS: bool, const LOGICAL: bool>(
        core: &mut Core<impl Bus>,
        value: u32,
        shift_amount: u32,
    ) -> u32 {
        let shift_amount = if shift_amount == 0 { 32 } else { shift_amount };

        if SET_FLAGS && LOGICAL {
            core.cpsr.c = (value & 0x8000_0000u32.rotate_left(shift_amount)) != 0;
        }

        let result = value >> shift_amount;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

use super::{Bus, Core, Size};

pub trait ShiftOperator {
    const NAME: &'static str;
    fn apply<T: Size>(core: &mut Core<impl Bus>, amount: u32, value: T) -> T;
}

pub struct Lsr;

impl ShiftOperator for Lsr {
    const NAME: &'static str = "LSR";

    fn apply<T: Size>(core: &mut Core<impl Bus>, amount: u32, value: T) -> T {
        if amount == 0 {
            core.set_ccr(|flags| {
                flags.set_nz(value);
                flags.c = false;
            });
            return value;
        }

        let carries = (value & T::one().wrapping_shl(amount - 1)) != T::zero();
        let result = value.wrapping_shr(amount);
        core.set_ccr(|flags| {
            flags.set_nz(result);
            flags.c = carries;
            flags.x = carries;
        });
        result
    }
}

use super::{Bus, Core, Size};

pub trait ShiftOperator {
    const NAME: &'static str;
    fn apply<T: Size>(core: &mut Core<impl Bus>, amount: u32, value: T) -> T;
}

pub struct Roxl;

impl ShiftOperator for Roxl {
    const NAME: &'static str = "ROXL";

    fn apply<T: Size>(core: &mut Core<impl Bus>, amount: u32, value: T) -> T {
        let extend = T::from(core.flags.x as u8).unwrap();
        let mut result = value.wrapping_shl(amount);

        core.set_ccr(|flags| {
            if amount != 0 {
                result = result | extend.wrapping_shl(amount - 1) | value.wrapping_shr(amount + 1);
                flags.c = (value & T::SIGN_BIT.wrapping_shr(amount - 1)) != T::zero();
            } else {
                flags.c = false;
            }

            flags.set_nz(result);
            flags.v = false;
            flags.x = flags.c;
        });

        result
    }
}

pub struct Lsr;

impl ShiftOperator for Lsr {
    const NAME: &'static str = "LSR";

    fn apply<T: Size>(core: &mut Core<impl Bus>, amount: u32, value: T) -> T {
        let result = value.wrapping_shr(amount);

        core.set_ccr(|flags| {
            if amount != 0 {
                flags.c = (value & T::one().wrapping_shl(amount - 1)) != T::zero();
                flags.x = flags.c;
            } else {
                flags.c = false;
                // X flag is unaffected
            }

            flags.set_nz(result);
            flags.v = false;
        });

        result
    }
}

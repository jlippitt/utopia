use super::{Bus, Core, Size};

pub trait BinaryOperator {
    const NAME: &'static str;
    fn apply<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) -> T;
}

pub struct Add;

impl BinaryOperator for Add {
    const NAME: &'static str = "ADD";

    fn apply<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) -> T {
        let result = lhs.wrapping_add(&rhs);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ result);
        core.set_ccr(|flags| {
            flags.set_nz(result);
            flags.v = (overflow & T::SIGN_BIT) != T::zero();
            flags.c = ((carries ^ overflow) & T::SIGN_BIT) != T::zero();
            flags.x = flags.c;
        });
        result
    }
}

pub struct Sub;

impl BinaryOperator for Sub {
    const NAME: &'static str = "SUB";

    fn apply<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) -> T {
        let result = lhs.wrapping_sub(&rhs);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ lhs);
        core.set_ccr(|flags| {
            flags.set_nz(result);
            flags.v = (overflow & T::SIGN_BIT) != T::zero();
            flags.c = ((carries ^ overflow) & T::SIGN_BIT) != T::zero();
            flags.x = flags.c;
        });
        result
    }
}

pub struct And;

impl BinaryOperator for And {
    const NAME: &'static str = "AND";

    fn apply<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) -> T {
        let result = lhs & rhs;
        core.set_ccr(|flags| {
            flags.set_nz(result);
            flags.v = false;
            flags.c = false;
        });
        result
    }
}

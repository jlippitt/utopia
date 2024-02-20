use super::{Bus, Core, Size};

pub trait Operator {
    const NAME: &'static str;
    fn apply<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) -> T;
}

pub struct And;

impl Operator for And {
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

pub struct Add;

impl Operator for Add {
    const NAME: &'static str = "ADD";

    fn apply<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) -> T {
        let carry = T::from(core.flags.c as u8).unwrap();
        let result = lhs.wrapping_add(&rhs).wrapping_add(&carry);
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

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
            flags.v = 0;
            flags.c = false;
        });
        result
    }
}

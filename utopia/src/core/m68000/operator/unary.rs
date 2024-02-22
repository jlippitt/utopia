use super::{Bus, Core, Size};

pub trait UnaryOperator {
    const NAME: &'static str;
    fn apply<T: Size>(core: &mut Core<impl Bus>, value: T) -> T;
}

pub struct Clr;

impl UnaryOperator for Clr {
    const NAME: &'static str = "CLR";

    fn apply<T: Size>(core: &mut Core<impl Bus>, _value: T) -> T {
        core.set_ccr(|flags| {
            flags.n = false;
            flags.z = true;
            flags.v = false;
            flags.c = false;
        });
        T::zero()
    }
}

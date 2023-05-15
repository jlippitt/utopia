use super::super::{Bus, Core};

pub trait ReadOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, value: u8);
}

fn compare(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) {
    let (result, borrow) = lhs.overflowing_sub(rhs);
    core.set_nz(result);
    core.flags.c = !borrow;
}

pub struct Lda;

impl ReadOperator for Lda {
    const NAME: &'static str = "LDA";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        core.a = value;
        core.set_nz(value);
    }
}

pub struct Ldx;

impl ReadOperator for Ldx {
    const NAME: &'static str = "LDX";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        core.x = value;
        core.set_nz(value);
    }
}

pub struct Ldy;

impl ReadOperator for Ldy {
    const NAME: &'static str = "LDY";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        core.y = value;
        core.set_nz(value);
    }
}

pub struct Cmp;

impl ReadOperator for Cmp {
    const NAME: &'static str = "CMP";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        compare(core, core.a, value);
    }
}

pub struct Cpx;

impl ReadOperator for Cpx {
    const NAME: &'static str = "CPX";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        compare(core, core.x, value);
    }
}

pub struct Cpy;

impl ReadOperator for Cpy {
    const NAME: &'static str = "CPY";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        compare(core, core.y, value);
    }
}

pub struct Bit;

impl ReadOperator for Bit {
    const NAME: &'static str = "BIT";

    fn apply(core: &mut Core<impl Bus>, value: u8) {
        core.flags.n = value;
        core.flags.v = value << 1;
        core.flags.z = value & core.a;
    }
}

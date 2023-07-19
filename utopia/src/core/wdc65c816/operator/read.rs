use super::super::{Bus, Core};

pub trait ReadOperator {
    const NAME: &'static str;
    fn apply8(core: &mut Core<impl Bus>, value: u8);
    fn apply16(core: &mut Core<impl Bus>, value: u16);
}

fn compare8(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) {
    let (result, borrow) = lhs.overflowing_sub(rhs);
    core.set_nz8(result);
    core.flags.c = !borrow;
}

fn compare16(core: &mut Core<impl Bus>, lhs: u16, rhs: u16) {
    let (result, borrow) = lhs.overflowing_sub(rhs);
    core.set_nz16(result);
    core.flags.c = !borrow;
}

pub struct Lda;

impl ReadOperator for Lda {
    const NAME: &'static str = "LDA";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        core.a = (core.a & 0xff00) | (value as u16);
        core.set_nz8(value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        core.a = value;
        core.set_nz16(value);
    }
}

pub struct Ldx;

impl ReadOperator for Ldx {
    const NAME: &'static str = "LDX";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        core.x = value as u16;
        core.set_nz8(value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        core.x = value;
        core.set_nz16(value);
    }
}

pub struct Ldy;

impl ReadOperator for Ldy {
    const NAME: &'static str = "LDY";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        core.y = value as u16;
        core.set_nz8(value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        core.y = value;
        core.set_nz16(value);
    }
}

pub struct Cmp;

impl ReadOperator for Cmp {
    const NAME: &'static str = "CMP";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        compare8(core, core.a as u8, value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        compare16(core, core.a, value);
    }
}

pub struct Cpx;

impl ReadOperator for Cpx {
    const NAME: &'static str = "CPX";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        compare8(core, core.x as u8, value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        compare16(core, core.x, value);
    }
}

pub struct Cpy;

impl ReadOperator for Cpy {
    const NAME: &'static str = "CPY";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        compare8(core, core.y as u8, value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        compare16(core, core.y, value);
    }
}

pub struct Ora;

impl ReadOperator for Ora {
    const NAME: &'static str = "ORA";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = core.a as u8 | value;
        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        core.a |= value;
        core.set_nz16(core.a);
    }
}

pub struct And;

impl ReadOperator for And {
    const NAME: &'static str = "AND";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = core.a as u8 & value;
        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        core.a &= value;
        core.set_nz16(core.a);
    }
}

pub struct Eor;

impl ReadOperator for Eor {
    const NAME: &'static str = "EOR";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = core.a as u8 ^ value;
        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        core.a ^= value;
        core.set_nz16(core.a);
    }
}

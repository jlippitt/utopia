use super::super::{Bus, Core};

pub trait ReadOperator {
    const NAME: &'static str;
    fn apply8(core: &mut Core<impl Bus>, value: u8);
    fn apply16(core: &mut Core<impl Bus>, value: u16);
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

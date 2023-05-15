use super::super::{Bus, Core};

pub trait ReadOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, value: u8);
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

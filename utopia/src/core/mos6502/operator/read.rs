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

use super::super::{Bus, Core};

pub trait WriteOperator {
    const NAME: &'static str;
    fn apply(core: &Core<impl Bus>) -> u8;
}

pub struct Sta;

impl WriteOperator for Sta {
    const NAME: &'static str = "STA";

    fn apply(core: &Core<impl Bus>) -> u8 {
        core.a
    }
}

pub struct Stx;

impl WriteOperator for Stx {
    const NAME: &'static str = "STX";

    fn apply(core: &Core<impl Bus>) -> u8 {
        core.x
    }
}

pub struct Sty;

impl WriteOperator for Sty {
    const NAME: &'static str = "STY";

    fn apply(core: &Core<impl Bus>) -> u8 {
        core.y
    }
}

pub struct Stz;

impl WriteOperator for Stz {
    const NAME: &'static str = "STZ";

    fn apply(_core: &Core<impl Bus>) -> u8 {
        0
    }
}

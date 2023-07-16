use super::super::{Bus, Core};

pub trait WriteOperator {
    const NAME: &'static str;
    fn apply8(core: &Core<impl Bus>) -> u8;
    fn apply16(core: &Core<impl Bus>) -> u16;
}

pub struct Sta;

impl WriteOperator for Sta {
    const NAME: &'static str = "STA";

    fn apply8(core: &Core<impl Bus>) -> u8 {
        core.a as u8
    }

    fn apply16(core: &Core<impl Bus>) -> u16 {
        core.a
    }
}

pub struct Stx;

impl WriteOperator for Stx {
    const NAME: &'static str = "STX";

    fn apply8(core: &Core<impl Bus>) -> u8 {
        core.x as u8
    }

    fn apply16(core: &Core<impl Bus>) -> u16 {
        core.x
    }
}

pub struct Sty;

impl WriteOperator for Sty {
    const NAME: &'static str = "STY";

    fn apply8(core: &Core<impl Bus>) -> u8 {
        core.y as u8
    }

    fn apply16(core: &Core<impl Bus>) -> u16 {
        core.y
    }
}

pub struct Stz;

impl WriteOperator for Stz {
    const NAME: &'static str = "STZ";

    fn apply8(_core: &Core<impl Bus>) -> u8 {
        0
    }

    fn apply16(_core: &Core<impl Bus>) -> u16 {
        0
    }
}

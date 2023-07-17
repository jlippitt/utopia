use super::super::{Bus, Core};

pub trait ReadOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8;
}

pub struct Mov;

impl ReadOperator for Mov {
    const NAME: &'static str = "MOV";

    fn apply(core: &mut Core<impl Bus>, value: u8) -> u8 {
        core.set_nz(value);
        value
    }
}

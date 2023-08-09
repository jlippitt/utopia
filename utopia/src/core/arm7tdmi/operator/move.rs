use super::super::{Bus, Core};

pub trait MoveOperator {
    const NAME: &'static str;
    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, value: u32) -> u32;
}

pub struct Mov;

impl MoveOperator for Mov {
    const NAME: &'static str = "MOV";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, value: u32) -> u32 {
        if SET_FLAGS {
            core.set_nz(value);
        }

        value
    }
}

pub struct Mvn;

impl MoveOperator for Mvn {
    const NAME: &'static str = "MVN";

    fn apply<const SET_FLAGS: bool>(core: &mut Core<impl Bus>, value: u32) -> u32 {
        let result = !value;

        if SET_FLAGS {
            core.set_nz(result);
        }

        result
    }
}

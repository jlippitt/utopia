use super::super::{Bus, Core};

pub trait BinaryOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8;
}

pub struct Mov;

impl BinaryOperator for Mov {
    const NAME: &'static str = "MOV";

    fn apply(core: &mut Core<impl Bus>, _lhs: u8, rhs: u8) -> u8 {
        core.set_nz(rhs);
        rhs
    }
}

pub struct Adc;

impl BinaryOperator for Adc {
    const NAME: &'static str = "ADC";

    fn apply(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8 {
        let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u8);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ result);
        core.set_nz(result);
        core.flags.c = ((carries ^ overflow) & 0x80) != 0;
        core.flags.v = overflow;
        result
    }
}

pub struct Sbc;

impl BinaryOperator for Sbc {
    const NAME: &'static str = "SBC";

    fn apply(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8 {
        Adc::apply(core, lhs, !rhs)
    }
}

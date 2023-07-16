use super::super::{Bus, Core};
use super::ReadOperator;

fn binary_add8(core: &mut Core<impl Bus>, rhs: u8) {
    let lhs = core.a as u8;
    let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u8);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.a = (core.a & 0xff00) | (result as u16);
    core.set_nz8(result);
    core.flags.v = (overflow & 0x80) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
}

fn binary_add16(core: &mut Core<impl Bus>, rhs: u16) {
    let lhs = core.a;
    let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u16);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.a = result;
    core.set_nz16(result);
    core.flags.v = (overflow & 0x8000) != 0;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
}

pub struct Adc;

impl ReadOperator for Adc {
    const NAME: &'static str = "ADC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        if core.flags.d {
            todo!("Decimal mode");
        }

        binary_add8(core, value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        if core.flags.d {
            todo!("Decimal mode");
        }

        binary_add16(core, value);
    }
}

pub struct Sbc;

impl ReadOperator for Sbc {
    const NAME: &'static str = "SBC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        if core.flags.d {
            todo!("Decimal mode");
        }

        binary_add8(core, !value);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        if core.flags.d {
            todo!("Decimal mode");
        }

        binary_add16(core, !value);
    }
}

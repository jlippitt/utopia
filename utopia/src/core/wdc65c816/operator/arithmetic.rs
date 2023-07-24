use super::super::{Bus, Core};
use super::ReadOperator;
use tracing::warn;

pub struct Adc;

impl ReadOperator for Adc {
    const NAME: &'static str = "ADC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = if core.flags.d {
            decimal_add8(core, core.a as u8, value)
        } else {
            binary_add8(core, core.a as u8, value)
        };

        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        let result = if core.flags.d {
            decimal_add16(core, core.a, value)
        } else {
            binary_add16(core, core.a, value)
        };

        core.a = result;
        core.set_nz16(result);
    }
}

pub struct Sbc;

impl ReadOperator for Sbc {
    const NAME: &'static str = "SBC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        if core.flags.d {
            warn!("Decimal mode not yet implemented");
        }

        let result = binary_add8(core, core.a as u8, !value);
        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        if core.flags.d {
            warn!("Decimal mode not yet implemented");
        }

        let result = binary_add16(core, core.a, !value);
        core.a = result;
        core.set_nz16(result);
    }
}

fn binary_add8(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8 {
    let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u8);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.flags.v = (overflow & 0x80) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

fn binary_add16(core: &mut Core<impl Bus>, lhs: u16, rhs: u16) -> u16 {
    let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u16);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.flags.v = (overflow & 0x8000) != 0;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
    result
}

fn decimal_add8(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8 {
    // Digit 0
    let mut result = (lhs & 0x0f)
        .wrapping_add(rhs & 0x0f)
        .wrapping_add(core.flags.c as u8);

    if result > 0x09 {
        result = result.wrapping_add(0x06);
    }

    core.flags.c = result > 0x0f;

    // Digit 1
    result = (result & 0x0f)
        .wrapping_add(lhs & 0xf0)
        .wrapping_add(rhs & 0xf0)
        .wrapping_add((core.flags.c as u8) << 4);

    core.flags.v = ((lhs ^ result) & (rhs ^ result) & 0x80) != 0;

    if result > 0x9f {
        result = result.wrapping_add(0x60);
    }

    core.flags.c = result < lhs;

    result
}

fn decimal_add16(core: &mut Core<impl Bus>, lhs: u16, rhs: u16) -> u16 {
    // Digit 0
    let mut result = (lhs & 0x000f)
        .wrapping_add(rhs & 0x000f)
        .wrapping_add(core.flags.c as u16);

    if result > 0x0009 {
        result = result.wrapping_add(0x0006);
    }

    core.flags.c = result > 0x000f;

    // Digit 1
    result = (result & 0x000f)
        .wrapping_add(lhs & 0x00f0)
        .wrapping_add(rhs & 0x00f0)
        .wrapping_add((core.flags.c as u16) << 4);

    if result > 0x009f {
        result = result.wrapping_add(0x0060);
    }

    core.flags.c = result > 0x00ff;

    // Digit 2
    result = (result & 0x00ff)
        .wrapping_add(lhs & 0x0f00)
        .wrapping_add(rhs & 0x0f00)
        .wrapping_add((core.flags.c as u16) << 8);

    if result > 0x09ff {
        result = result.wrapping_add(0x0600);
    }

    core.flags.c = result > 0x0fff;

    // Digit 3
    result = (result & 0x0fff)
        .wrapping_add(lhs & 0xf000)
        .wrapping_add(rhs & 0xf000)
        .wrapping_add((core.flags.c as u16) << 12);

    core.flags.v = ((lhs ^ result) & (rhs ^ result) & 0x8000) != 0;

    if result > 0x9fff {
        result = result.wrapping_add(0x6000);
    }

    core.flags.c = result < lhs;

    result
}

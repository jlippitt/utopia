use super::super::{Bus, Core};
use super::ReadOperator;

pub struct Adc;

impl ReadOperator for Adc {
    const NAME: &'static str = "ADC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = if core.flags.d {
            decimal_add8::<false>(core, core.a as u8, value)
        } else {
            let result = (core.a as u8)
                .wrapping_add(value)
                .wrapping_add(core.flags.c as u8);
            set_vc8(core, core.a as u8, value, result);
            result
        };

        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        let result = if core.flags.d {
            decimal_add16::<false>(core, core.a, value)
        } else {
            let result = core.a.wrapping_add(value).wrapping_add(core.flags.c as u16);
            set_vc16(core, core.a, value, result);
            result
        };

        core.a = result;
        core.set_nz16(result);
    }
}

pub struct Sbc;

impl ReadOperator for Sbc {
    const NAME: &'static str = "SBC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = if core.flags.d {
            decimal_add8::<true>(core, core.a as u8, !value)
        } else {
            let result = (core.a as u8)
                .wrapping_add(!value)
                .wrapping_add(core.flags.c as u8);
            set_vc8(core, core.a as u8, !value, result);
            result
        };

        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        let result = if core.flags.d {
            decimal_add16::<true>(core, core.a, !value)
        } else {
            let result = core
                .a
                .wrapping_add(!value)
                .wrapping_add(core.flags.c as u16);
            set_vc16(core, core.a, !value, result);
            result
        };

        core.a = result;
        core.set_nz16(result);
    }
}

fn set_vc8(core: &mut Core<impl Bus>, lhs: u8, rhs: u8, result: u8) {
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.flags.v = (overflow & 0x80) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
}

fn set_vc16(core: &mut Core<impl Bus>, lhs: u16, rhs: u16, result: u16) {
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.flags.v = (overflow & 0x8000) != 0;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
}

fn decimal_add8<const SBC: bool>(core: &mut Core<impl Bus>, lhs: u8, rhs: u8) -> u8 {
    // Digit 0
    let mut result = (lhs & 0x0f)
        .wrapping_add(rhs & 0x0f)
        .wrapping_add(core.flags.c as u8);

    if SBC {
        if result <= 0x0f {
            result = result.wrapping_sub(0x06);
        }
    } else {
        if result > 0x09 {
            result = result.wrapping_add(0x06);
        }
    }

    core.flags.c = result > 0x0f;

    // Digit 1
    result = (result & 0x0f)
        .wrapping_add(lhs & 0xf0)
        .wrapping_add(rhs & 0xf0)
        .wrapping_add((core.flags.c as u8) << 4);

    set_vc8(core, lhs, rhs, result);

    if SBC {
        if result >= lhs {
            result = result.wrapping_sub(0x60);
        }
    } else {
        if result > 0x9f || core.flags.c {
            let (new_result, carry) = result.overflowing_add(0x60);
            result = new_result;
            core.flags.c |= carry;
        }
    }

    result
}

fn decimal_add16<const SBC: bool>(core: &mut Core<impl Bus>, lhs: u16, rhs: u16) -> u16 {
    // Digit 0
    let mut result = (lhs & 0x000f)
        .wrapping_add(rhs & 0x000f)
        .wrapping_add(core.flags.c as u16);

    if SBC {
        if result <= 0x000f {
            result = result.wrapping_sub(0x0006);
        }
    } else {
        if result > 0x0009 {
            result = result.wrapping_add(0x0006);
        }
    }

    core.flags.c = result > 0x000f;

    // Digit 1
    result = (result & 0x000f)
        .wrapping_add(lhs & 0x00f0)
        .wrapping_add(rhs & 0x00f0)
        .wrapping_add((core.flags.c as u16) << 4);

    if SBC {
        if result <= 0x00ff {
            result = result.wrapping_sub(0x0060);
        }
    } else {
        if result > 0x009f {
            result = result.wrapping_add(0x0060);
        }
    }

    core.flags.c = result > 0x00ff;

    // Digit 2
    result = (result & 0x00ff)
        .wrapping_add(lhs & 0x0f00)
        .wrapping_add(rhs & 0x0f00)
        .wrapping_add((core.flags.c as u16) << 8);

    if SBC {
        if result <= 0x0fff {
            result = result.wrapping_sub(0x0600);
        }
    } else {
        if result > 0x09ff {
            result = result.wrapping_add(0x0600);
        }
    }

    core.flags.c = result > 0x0fff;

    // Digit 3
    result = (result & 0x0fff)
        .wrapping_add(lhs & 0xf000)
        .wrapping_add(rhs & 0xf000)
        .wrapping_add((core.flags.c as u16) << 12);

    set_vc16(core, lhs, rhs, result);

    if SBC {
        if result >= lhs {
            result = result.wrapping_sub(0x6000);
        }
    } else {
        if result > 0x9fff || core.flags.c {
            let (new_result, carry) = result.overflowing_add(0x6000);
            result = new_result;
            core.flags.c |= carry;
        }
    }

    result
}

use super::super::{Bus, Core};
use super::ReadOperator;

pub struct Adc;

impl ReadOperator for Adc {
    const NAME: &'static str = "ADC";

    fn apply8(core: &mut Core<impl Bus>, value: u8) {
        let result = if core.flags.d {
            decimal_add8::<false>(core, value as i32)
        } else {
            binary_add8(core, value)
        };

        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        let result = if core.flags.d {
            decimal_add16::<false>(core, value as i32)
        } else {
            binary_add16(core, value)
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
            decimal_add8::<true>(core, !value as i32)
        } else {
            binary_add8(core, !value)
        };

        core.a = (core.a & 0xff00) | (result as u16);
        core.set_nz8(result);
    }

    fn apply16(core: &mut Core<impl Bus>, value: u16) {
        let result = if core.flags.d {
            decimal_add16::<true>(core, !value as i32)
        } else {
            binary_add16(core, !value)
        };

        core.a = result;
        core.set_nz16(result);
    }
}

fn binary_add8(core: &mut Core<impl Bus>, rhs: u8) -> u8 {
    let lhs = core.a as u8;
    let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u8);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.flags.v = (overflow & 0x80) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

fn binary_add16(core: &mut Core<impl Bus>, rhs: u16) -> u16 {
    let lhs = core.a;
    let result = lhs.wrapping_add(rhs).wrapping_add(core.flags.c as u16);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ result);
    core.flags.v = (overflow & 0x8000) != 0;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
    result
}

fn decimal_add8<const SBC: bool>(core: &mut Core<impl Bus>, rhs: i32) -> u8 {
    let lhs = core.a as i32;

    // Digit 0
    let mut result = (lhs & 0x0f) + (rhs & 0x0f) + (core.flags.c as i32);

    if SBC {
        if result <= 0x0f {
            result -= 0x06;
        }
    } else {
        if result > 0x09 {
            result += 0x06;
        }
    }

    core.flags.c = result > 0x0f;

    // Digit 1
    result = (result & 0x0f) + (lhs & 0xf0) + (rhs & 0xf0) + ((core.flags.c as i32) << 4);

    core.flags.v = ((lhs ^ result) & (rhs ^ result) & 0x80) != 0;

    if SBC {
        if result <= 0xff {
            result -= 0x60;
        }
    } else {
        if result > 0x9f {
            result += 0x60;
        }
    }

    core.flags.c = result > 0xff;

    result as u8
}

fn decimal_add16<const SBC: bool>(core: &mut Core<impl Bus>, rhs: i32) -> u16 {
    let lhs = core.a as i32;

    // Digit 0
    let mut result = (lhs & 0x000f) + (rhs & 0x000f) + (core.flags.c as i32);

    if SBC {
        if result <= 0x000f {
            result -= 0x0006;
        }
    } else {
        if result > 0x0009 {
            result += 0x0006;
        }
    }

    core.flags.c = result > 0x000f;

    // Digit 1
    result = (result & 0x000f) + (lhs & 0x00f0) + (rhs & 0x00f0) + ((core.flags.c as i32) << 4);

    if SBC {
        if result <= 0x00ff {
            result -= 0x0060;
        }
    } else {
        if result > 0x009f {
            result += 0x0060;
        }
    }

    core.flags.c = result > 0x00ff;

    // Digit 2
    result = (result & 0x00ff) + (lhs & 0x0f00) + (rhs & 0x0f00) + ((core.flags.c as i32) << 8);

    if SBC {
        if result <= 0x0fff {
            result -= 0x0600;
        }
    } else {
        if result > 0x09ff {
            result += 0x0600;
        }
    }

    core.flags.c = result > 0x0fff;

    // Digit 3
    result = (result & 0x0fff) + (lhs & 0xf000) + (rhs & 0xf000) + ((core.flags.c as i32) << 12);

    core.flags.v = ((lhs ^ result) & (rhs ^ result) & 0x8000) != 0;

    if SBC {
        if result <= 0xffff {
            result -= 0x6000;
        }
    } else {
        if result > 0x9fff {
            result += 0x6000;
        }
    }

    core.flags.c = result > 0xffff;

    result as u16
}

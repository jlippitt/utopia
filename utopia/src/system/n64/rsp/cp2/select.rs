use super::compute::compute;
use super::Cp2;
use crate::n64::mips::{Bus, Core};

pub fn vlt(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    select("VLT", core, word, |cp2, index, lhs, rhs| {
        (lhs as i16) < (rhs as i16)
            || (lhs == rhs && cp2.carry[index] && !cp2.compare_extension[index])
    });
}

pub fn veq(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    select("VEQ", core, word, |cp2, index, lhs, rhs| {
        lhs == rhs && (!cp2.carry[index] || cp2.compare_extension[index])
    });
}

pub fn vne(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    select("VNE", core, word, |cp2, index, lhs, rhs| {
        lhs != rhs || (cp2.carry[index] && !cp2.compare_extension[index])
    });
}

pub fn vge(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    select("VGE", core, word, |cp2, index, lhs, rhs| {
        (lhs as i16) > (rhs as i16)
            || (lhs == rhs && (!cp2.carry[index] || cp2.compare_extension[index]))
    });
}

pub fn vcl(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VCL", core, word, |cp2, index, acc, lhs, rhs| {
        let result = if cp2.carry[index] {
            let lt = if cp2.not_equal[index] {
                cp2.compare[index]
            } else {
                let result = lhs as u32 + rhs as u32;

                let lt = if cp2.compare_extension[index] {
                    result <= 0x10000
                } else {
                    result == 0
                };

                cp2.compare.set(index, lt);
                lt
            };

            if lt {
                (rhs as i16).wrapping_neg() as u16
            } else {
                lhs
            }
        } else {
            let ge = if cp2.not_equal[index] {
                cp2.clip_compare[index]
            } else {
                let ge = lhs >= rhs;
                cp2.clip_compare.set(index, ge);
                ge
            };

            if ge {
                rhs
            } else {
                lhs
            }
        };

        *acc = result as u64;
        result
    });

    core.cp2_mut().not_equal.fill(false);
    core.cp2_mut().carry.fill(false);
    core.cp2_mut().compare_extension.fill(false);
}

pub fn vch(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VCH", core, word, |cp2, index, acc, lhs, rhs| {
        let carry = (lhs as i16 ^ rhs as i16) < 0;
        cp2.carry.set(index, carry);

        let result = if carry {
            let value = (lhs as i16).wrapping_add(rhs as i16);
            let lt = value <= 0;

            cp2.not_equal.set(index, value != 0 && (lhs != !rhs));
            cp2.compare.set(index, lt);
            cp2.clip_compare.set(index, (rhs as i16) < 0);
            cp2.compare_extension.set(index, value == -1);

            if lt {
                (rhs as i16).wrapping_neg() as u16
            } else {
                lhs
            }
        } else {
            let value = (lhs as i16).wrapping_sub(rhs as i16);
            let ge = value >= 0;

            cp2.not_equal.set(index, value != 0 && (lhs != !rhs));
            cp2.compare.set(index, (rhs as i16) < 0);
            cp2.clip_compare.set(index, ge);
            cp2.compare_extension.set(index, false);

            if ge {
                rhs
            } else {
                lhs
            }
        };

        *acc = result as u64;
        result
    });
}

pub fn vcr(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VCR", core, word, |cp2, index, acc, lhs, rhs| {
        let result = if (lhs as i16 ^ rhs as i16) < 0 {
            cp2.clip_compare.set(index, (rhs as i16) < 0);
            let lt = (lhs as i16).wrapping_add(rhs as i16) < 0;
            cp2.compare.set(index, lt);

            if lt {
                !rhs
            } else {
                lhs
            }
        } else {
            cp2.compare.set(index, (rhs as i16) < 0);

            let ge = (lhs as i16).wrapping_sub(rhs as i16) >= 0;
            cp2.clip_compare.set(index, ge);

            if ge {
                rhs
            } else {
                lhs
            }
        };

        *acc = result as u64;
        result
    });
}

pub fn vmrg(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMRG", core, word, |cp2, index, acc, lhs, rhs| {
        let result = if cp2.compare[index] { lhs } else { rhs };
        *acc = result as u64;
        result
    });

    core.cp2_mut().not_equal.fill(false);
    core.cp2_mut().carry.fill(false);
}

fn select(
    name: &'static str,
    core: &mut Core<impl Bus<Cp2 = Cp2>>,
    word: u32,
    cb: impl Fn(&mut Cp2, usize, u16, u16) -> bool,
) {
    compute(name, core, word, |cp2, index, acc, lhs, rhs| {
        let condition = cb(cp2, index ^ 7, lhs, rhs);
        cp2.compare.set(index, condition);
        let result = if condition { lhs } else { rhs };
        *acc = result as u64;
        result
    });

    core.cp2_mut().not_equal.fill(false);
    core.cp2_mut().carry.fill(false);
    core.cp2_mut().compare_extension.fill(false);
}

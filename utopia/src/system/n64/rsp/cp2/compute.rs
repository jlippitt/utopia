use super::Cp2;
use crate::core::mips::{Bus, Core};
use bitfield_struct::bitfield;
use std::cmp::Ordering;
use tracing::trace;

#[bitfield(u32)]
pub struct Compute {
    #[bits(6)]
    opcode: u32,
    #[bits(5)]
    vd: usize,
    #[bits(5)]
    vs: usize,
    #[bits(5)]
    vt: usize,
    #[bits(4)]
    element: usize,
    #[bits(7)]
    __: u32,
}

pub fn vmulf(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMULF", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs as i16 as i32 * rhs as i16 as i32;
        *acc = ((result << 1).wrapping_add(0x8000)) as i64 as u64;
        clamp_signed((*acc >> 16) as i32) as u16
    });
}

pub fn vmulu(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMULU", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs as i16 as i32 * rhs as i16 as i32;
        *acc = ((result << 1).wrapping_add(0x8000)) as i64 as u64;
        clamp_unsigned((*acc >> 16) as i32)
    });
}

pub fn vmacf(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMACF", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs as i16 as i32 * rhs as i16 as i32;
        *acc = (*acc as i64 + (result << 1) as i64) as u64;
        clamp_signed((*acc >> 16) as i32) as u16
    });
}

pub fn vmacu(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMACU", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs as i16 as i32 * rhs as i16 as i32;
        *acc = (*acc as i64 + (result << 1) as i64) as u64;
        clamp_unsigned((*acc >> 16) as i32)
    });
}

pub fn vmudl(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMUDL", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as u32).wrapping_mul(rhs as u32);
        *acc = (result >> 16) as u64;
        *acc as i64 as i16 as u16
    });
}

pub fn vmudm(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMUDM", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as i16 as u32).wrapping_mul(rhs as u32);
        *acc = result as i32 as i64 as u64;
        clamp_signed((*acc >> 16) as i32) as u16
    });
}

pub fn vmudn(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMUDN", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as u32).wrapping_mul(rhs as i16 as u32);
        *acc = result as i32 as i64 as u64;
        *acc as i64 as i16 as u16
    });
}

pub fn vmudh(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMUDH", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as i16 as u32).wrapping_mul(rhs as i16 as u32);
        *acc = ((result as i32 as i64) << 16) as u64;
        clamp_signed((*acc >> 16) as i32) as u16
    });
}

pub fn vmadl(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMADL", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as u32).wrapping_mul(rhs as u32);
        *acc = (*acc as i64 + ((result >> 16) as i64)) as u64;
        *acc as i64 as i16 as u16
    });
}

pub fn vmadm(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMADM", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as i16 as u32).wrapping_mul(rhs as u32);
        *acc = (*acc as i64 + result as i32 as i64) as u64;
        clamp_signed((*acc >> 16) as i32) as u16
    });
}

pub fn vmadn(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMADN", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as u32).wrapping_mul(rhs as i16 as u32);
        *acc = (*acc as i64 + result as i32 as i64) as u64;
        *acc as i64 as i16 as u16
    });
}

pub fn vmadh(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VMADH", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = (lhs as i16 as u32).wrapping_mul(rhs as i16 as u32);
        *acc = (*acc as i64 + ((result as i32 as i64) << 16)) as u64;
        clamp_signed((*acc >> 16) as i32) as u16
    });
}

pub fn vadd(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VADD", core, word, |cp2, index, acc, lhs, rhs| {
        let result = lhs as i16 as i32 + rhs as i16 as i32 + cp2.carry[index] as i16 as i32;
        *acc = result as u16 as u64;
        clamp_signed(result) as u16
    });

    core.cp2_mut().not_equal.fill(false);
    core.cp2_mut().carry.fill(false);
}

pub fn vaddc(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VADDC", core, word, |cp2, index, acc, lhs, rhs| {
        let result = lhs as u32 + rhs as u32;
        *acc = result as u16 as u64;
        cp2.carry.set(index, (result & 0x0001_0000) != 0);
        result as u16
    });

    core.cp2_mut().not_equal.fill(false);
}

pub fn vsub(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VSUB", core, word, |cp2, index, acc, lhs, rhs| {
        let result = lhs as i16 as i32 - rhs as i16 as i32 - cp2.carry[index] as i16 as i32;
        *acc = result as u16 as u64;
        clamp_signed(result) as u16
    });

    core.cp2_mut().not_equal.fill(false);
    core.cp2_mut().carry.fill(false);
}

pub fn vsubc(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VSUBC", core, word, |cp2, index, acc, lhs, rhs| {
        let result = lhs as i32 - rhs as i32;
        *acc = result as u16 as u64;
        cp2.carry.set(index, result < 0);
        cp2.not_equal.set(index, result != 0);
        result as u16
    });
}

pub fn vabs(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VABS", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = match (lhs as i16).cmp(&0) {
            Ordering::Less => rhs.wrapping_neg(),
            Ordering::Equal => 0,
            Ordering::Greater => rhs,
        };
        *acc = result as u64;
        result
    });
}

pub fn vsar(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let op = decode("VSAR", core.pc(), word);
    let cp2 = core.cp2_mut();

    if (8..=10).contains(&op.element()) {
        let shift = 32 - ((op.element() - 8) * 16);
        let acc = cp2.acc_le();
        let result = std::array::from_fn(|index| (acc[index] >> shift) as u16);
        cp2.set_le(op.vd(), result);
    } else {
        cp2.set_le(op.vd(), [0; 8]);
    }
}

pub fn vand(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VAND", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs & rhs;
        *acc = result as u64;
        result
    });
}

pub fn vnand(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VNAND", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = !(lhs & rhs);
        *acc = result as u64;
        result
    });
}

pub fn vor(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VOR", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs | rhs;
        *acc = result as u64;
        result
    });
}

pub fn vnor(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VNOR", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = !(lhs | rhs);
        *acc = result as u64;
        result
    });
}

pub fn vxor(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VXOR", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = lhs ^ rhs;
        *acc = result as u64;
        result
    });
}

pub fn vnxor(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    compute("VNXOR", core, word, |_cp2, _index, acc, lhs, rhs| {
        let result = !(lhs ^ rhs);
        *acc = result as u64;
        result
    });
}

fn decode(name: &'static str, pc: u32, word: u32) -> Compute {
    let op = Compute::from(word);

    trace!(
        "{:08X} {} V{:02}, V{:02}, V{:02},E({})",
        pc,
        name,
        op.vd(),
        op.vs(),
        op.vt(),
        op.element(),
    );

    op
}

pub fn compute(
    name: &'static str,
    core: &mut Core<impl Bus<Cp2 = Cp2>>,
    word: u32,
    mut cb: impl FnMut(&mut Cp2, usize, &mut u64, u16, u16) -> u16,
) {
    let op = decode(name, core.pc(), word);
    let cp2 = core.cp2_mut();
    let lhs = cp2.get_le(op.vs());
    let rhs = cp2.broadcast_le(op.vt(), op.element());
    let mut acc = cp2.acc_le();
    let result =
        std::array::from_fn(|index| cb(cp2, index ^ 7, &mut acc[index], lhs[index], rhs[index]));
    cp2.set_le(op.vd(), result);
    cp2.set_acc_le(acc);
}

fn clamp_signed(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn clamp_unsigned(value: i32) -> u16 {
    if value >= i16::MAX as i32 {
        u16::MAX
    } else if value >= 0 {
        value as u16
    } else {
        0
    }
}

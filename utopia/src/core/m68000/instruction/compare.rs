use super::{AddressMode, Bus, Core, Size};
use tracing::trace;

fn compare<T: Size>(core: &mut Core<impl Bus>, lhs: T, rhs: T) {
    let result = lhs.wrapping_sub(&rhs);
    let carries = lhs ^ rhs ^ result;
    let overflow = (lhs ^ result) & (rhs ^ lhs);
    core.set_ccr(|flags| {
        flags.set_nz(result);
        flags.v = (overflow & T::SIGN_BIT) != T::zero();
        flags.c = ((carries ^ overflow) & T::SIGN_BIT) != T::zero();
    });
}

pub fn cmp<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    let dst = (word >> 9) & 7;
    trace!("CMP.{} {}, D{}", T::NAME, src, dst);
    let src_value: T = src.read(core);
    let dst_value = core.dreg(dst as usize);
    compare(core, dst_value, src_value);
}

pub fn cmpi<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);
    trace!("CMPI.{} #imm, {}", T::NAME, dst);
    let src_value: T = core.next();
    let dst_value = dst.read(core);
    compare(core, dst_value, src_value);
}

pub fn tst<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let operand = AddressMode::from(word);
    trace!("TST.{} {}", T::NAME, operand);
    let value: T = operand.read(core);
    core.set_ccr(|flags| {
        flags.set_nz(value);
        flags.v = false;
        flags.c = false;
    });
}

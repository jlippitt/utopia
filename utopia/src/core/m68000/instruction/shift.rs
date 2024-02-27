use super::operator::ShiftOperator;
use super::{Bus, Core, Size};
use tracing::trace;

pub fn shift_many_left<U: Size>(core: &mut Core<impl Bus>, word: u16) {
    use super::operator as op;

    match (word >> 3) & 7 {
        // 0b000 => shift_immediate::<op::Asl, U>(core, word),
        0b001 => shift_immediate::<op::Lsl, U>(core, word),
        0b010 => shift_immediate::<op::Roxl, U>(core, word),
        // 0b011 => shift_immediate::<op::Rol, U>(core, word),
        // 0b100 => shift_register::<op::Asl, U>(core, word),
        0b101 => shift_register::<op::Lsl, U>(core, word),
        0b110 => shift_register::<op::Roxl, U>(core, word),
        // 0b111 => shift_register::<op::Rol, U>(core, word),
        _ => unreachable!(),
    }
}

pub fn shift_many_right<U: Size>(core: &mut Core<impl Bus>, word: u16) {
    use super::operator as op;

    match (word >> 3) & 7 {
        // 0b000 => shift_immediate::<op::Asr, U>(core, word),
        0b001 => shift_immediate::<op::Lsr, U>(core, word),
        // 0b010 => shift_immediate::<op::Roxr, U>(core, word),
        // 0b011 => shift_immediate::<op::Ror, U>(core, word),
        // 0b100 => shift_register::<op::Asr, U>(core, word),
        0b101 => shift_register::<op::Lsr, U>(core, word),
        // 0b110 => shift_register::<op::Roxr, U>(core, word),
        // 0b111 => shift_register::<op::Ror, U>(core, word),
        _ => unreachable!(),
    }
}

fn shift_immediate<T: ShiftOperator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = word & 7;
    let src = ((((word >> 9) - 1) & 7) + 1) as u32;
    trace!("{}.{} #{}, D{}", T::NAME, U::NAME, src, dst);
    let value: U = core.dreg(dst as usize);
    let result = T::apply(core, src, value);
    core.set_dreg(dst as usize, result);
}

fn shift_register<T: ShiftOperator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = word & 7;
    let src = (word >> 9) & 7;
    trace!("{}.{} D{}, D{}", T::NAME, U::NAME, src, dst);
    let value: U = core.dreg(dst as usize);
    let result = T::apply(core, core.dreg::<u32>(src as usize) & 63, value);
    core.set_dreg(dst as usize, result);
}

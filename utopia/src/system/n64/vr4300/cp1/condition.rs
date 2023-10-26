use super::{Bus, Core, Cp1, Opcode};
use num_traits::Float;
use std::cmp::Ordering;
use tracing::trace;

const COND_NAMES: [&str; 16] = [
    "F", "UN", "EQ", "UEQ", "OLT", "ULT", "OLE", "ULE", "SF", "NGLE", "SEQ", "NGL", "LT", "NGE",
    "LE", "NGT",
];

pub fn c_s<const COND: u32>(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} C.{}.S F{}, F{}",
        core.pc(),
        COND_NAMES[COND as usize],
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.set_c(apply::<COND, f32>(cp1.gets(op.fs()), cp1.gets(op.ft())));
}

pub fn c_d<const COND: u32>(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} C.{}.S F{}, F{}",
        core.pc(),
        COND_NAMES[COND as usize],
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.set_c(apply::<COND, f64>(cp1.getd(op.fs()), cp1.getd(op.ft())));
}

fn apply<const COND: u32, T: Float>(lhs: T, rhs: T) -> bool {
    match COND & 7 {
        0 => false,
        1 => lhs.partial_cmp(&rhs).is_none(),
        2 => lhs == rhs,
        3 => matches!(lhs.partial_cmp(&rhs), None | Some(Ordering::Equal),),
        4 => lhs < rhs,
        5 => !matches!(
            lhs.partial_cmp(&rhs),
            Some(Ordering::Greater) | Some(Ordering::Equal),
        ),
        6 => lhs <= rhs,
        7 => !matches!(lhs.partial_cmp(&rhs), Some(Ordering::Greater),),
        _ => unreachable!(),
    }
}

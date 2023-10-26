use super::{Bus, Core, Cp1, IType};
use tracing::trace;

pub fn bc1f<const LIKELY: bool>(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BC1F{} {:+}",
        core.pc(),
        if LIKELY { "L" } else { "" },
        offset
    );

    let condition = !core.cp1().status.c();
    core.branch_if::<LIKELY>(condition, offset);
}

pub fn bc1t<const LIKELY: bool>(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = IType::from(word);
    let offset = (op.imm() as i16 as i32) << 2;

    trace!(
        "{:08X} BC1T{} {:+}",
        core.pc(),
        if LIKELY { "L" } else { "" },
        offset
    );

    let condition = core.cp1().status.c();
    core.branch_if::<LIKELY>(condition, offset);
}

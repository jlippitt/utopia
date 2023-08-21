use super::VectorUnit;
use crate::core::mips::{Bus, Core, REGS};
use tracing::debug;

pub fn sdv(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    base: usize,
    vt: usize,
    elem: usize,
    offset: i32,
) {
    debug_assert!((elem & 7) == 0);
    let offset = offset * 8;

    debug!(
        "{:08X} SDV $V{:02},E({}), {},{}",
        core.pc(),
        vt,
        elem >> 1,
        offset,
        REGS[base]
    );

    let address = core.get(base).wrapping_add(offset as u32);
    let result = core.cp2().getd(vt, elem);
    core.write_doubleword(address, result);
}

pub fn sqv(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    base: usize,
    vt: usize,
    elem: usize,
    offset: i32,
) {
    debug_assert!((elem & 15) == 0);
    let offset = offset * 16;

    debug!(
        "{:08X} SQV $V{:02},E({}), {},{}",
        core.pc(),
        vt,
        elem >> 1,
        offset,
        REGS[base]
    );

    let address = core.get(base).wrapping_add(offset as u32);
    let result = core.cp2().getq(vt, elem);
    core.write_doubleword(address, (result >> 64) as u64);
    core.write_doubleword(address.wrapping_add(8), result as u64);
}

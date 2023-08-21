use super::VectorUnit;
use crate::core::mips::{Bus, Core, REGS};
use tracing::debug;

pub fn ssv(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    base: usize,
    vt: usize,
    elem: usize,
    offset: i32,
) {
    debug_assert!((elem & 1) == 0);
    let offset = offset * 2;

    debug!(
        "{:08X} SSV $V{:02},E({}), {},{}",
        core.pc(),
        vt,
        elem >> 1,
        offset,
        REGS[base]
    );

    let address = core.get(base).wrapping_add(offset as u32);
    let result = core.cp2().get_h(vt, elem);
    core.write_halfword(address, result);
}

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
    let result = core.cp2().get_d(vt, elem);
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
    let result = core.cp2().get_q(vt, elem);
    core.write_doubleword(address, (result >> 64) as u64);
    core.write_doubleword(address.wrapping_add(8), result as u64);
}

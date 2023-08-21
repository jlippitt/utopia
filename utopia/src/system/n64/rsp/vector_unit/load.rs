use super::VectorUnit;
use crate::core::mips::{Bus, Core, REGS};
use tracing::debug;

pub fn ldv(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    base: usize,
    vt: usize,
    elem: usize,
    offset: i32,
) {
    debug_assert!((elem & 7) == 0);
    let offset = offset * 8;

    debug!(
        "{:08X} LDV $V{:02},E({}), {},{}",
        core.pc(),
        vt,
        elem >> 1,
        offset,
        REGS[base]
    );

    let address = core.get(base).wrapping_add(offset as u32);
    let result = core.read_doubleword(address);
    core.cp2_mut().setd(vt, elem, result);
}

pub fn lqv(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    base: usize,
    vt: usize,
    elem: usize,
    offset: i32,
) {
    debug_assert!((elem & 15) == 0);
    let offset = offset * 16;

    debug!(
        "{:08X} LQV $V{:02},E({}), {},{}",
        core.pc(),
        vt,
        elem >> 1,
        offset,
        REGS[base]
    );

    let address = core.get(base).wrapping_add(offset as u32);
    let high = core.read_doubleword(address) as u128;
    let low = core.read_doubleword(address.wrapping_add(8)) as u128;
    let result = (high << 64) | low;
    core.cp2_mut().setq(vt, elem, result);
}

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
    debug_assert!((elem & 1) == 0);

    debug!(
        "{:08X} LDV $V{:02},E({}) {}({})",
        core.pc(),
        vt,
        elem >> 1,
        offset,
        REGS[base]
    );

    let address = core.get(base).wrapping_add((offset * 8) as u32);
    let result = core.read_doubleword(address);
    core.cp2_mut().setd(vt, elem, result);
}

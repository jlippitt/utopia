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
    debug_assert!((elem & 1) == 0);
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

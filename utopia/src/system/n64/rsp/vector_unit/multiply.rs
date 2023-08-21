use super::VectorUnit;
use crate::core::mips::{Bus, Core};
use tracing::debug;

pub fn vmulf(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    elem: usize,
    vt: usize,
    vs: usize,
    vd: usize,
) {
    debug!(
        "{:08X} VMULF $V{:02}, $V{:02}, $V{:02},E({})",
        core.pc(),
        vd,
        vs,
        vt,
        elem,
    );

    debug_assert!((elem & 15) == 0);

    let lhs = core.cp2().getv(vs);
    let rhs = core.cp2().getv(vt);
    let mut result = [0; 8];

    for lane in 0..8 {
        let tmp = ((lhs[lane] as i32 * rhs[lane] as i32) << 1) + 32768;
        // TODO: Accumulator
        // TODO: Clamping
        result[lane] = (tmp >> 16) as u16;
    }

    core.cp2_mut().setv(vd, result);
}

use super::vector::Vector;
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
    let broadcast = elem.into();

    debug!(
        "{:08X} VMULF $V{:02}, $V{:02}, $V{:02}{}",
        core.pc(),
        vd,
        vs,
        vt,
        broadcast,
    );

    let lhs = core.cp2().getv(vs);
    let rhs = core.cp2().getv(vt).broadcast(broadcast);

    let result = Vector::from_fn(|lane| {
        let tmp = ((lhs[lane] as i16 as i32 * rhs[lane] as i16 as i32) << 1) + 32768;
        // TODO: Clamping
        core.cp2_mut().acc[lane] = tmp as i64 as u64;
        (tmp >> 16) as u16
    });

    core.cp2_mut().setv(vd, result);
}

pub fn vmacf(
    core: &mut Core<impl Bus<Cp2 = VectorUnit>>,
    elem: usize,
    vt: usize,
    vs: usize,
    vd: usize,
) {
    let broadcast = elem.into();

    debug!(
        "{:08X} VMACF $V{:02}, $V{:02}, $V{:02}{}",
        core.pc(),
        vd,
        vs,
        vt,
        broadcast,
    );

    let lhs = core.cp2().getv(vs);
    let rhs = core.cp2().getv(vt).broadcast(broadcast);

    let result = Vector::from_fn(|lane| {
        let tmp = (lhs[lane] as i16 as i32 * rhs[lane] as i16 as i32) << 1;
        // TODO: Clamping
        core.cp2_mut().accumulate(lane, tmp as i64 as u64);
        (tmp >> 16) as u16
    });

    core.cp2_mut().setv(vd, result);
}

use super::super::{Bus, Coprocessor0, Coprocessor2, Core, REGS};
use tracing::debug;

pub fn mfc0<T: Bus>(core: &mut Core<T>, _rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFC0 {}, {}", core.pc, REGS[rt], T::Cp0::REGS[rd]);
    core.set(rt, T::Cp0::get(core, rd));
}

pub fn mtc0<T: Bus>(core: &mut Core<T>, _rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MTC0 {}, {}", core.pc, REGS[rt], T::Cp0::REGS[rd]);
    T::Cp0::set(core, rd, core.get(rt));
}

pub fn mfc2<T: Bus>(core: &mut Core<T>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    let elem = sa as usize >> 1;

    debug!(
        "{:08X} MFC2 {}, {},E({})",
        core.pc,
        REGS[rt],
        T::Cp2::REGS[rd],
        elem
    );

    core.set(rt, T::Cp2::get(core, rd, elem));
}

pub fn mtc2<T: Bus>(core: &mut Core<T>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    let elem = sa as usize >> 1;

    debug!(
        "{:08X} MTC2 {}, {},E({})",
        core.pc,
        REGS[rt],
        T::Cp2::REGS[rd],
        elem
    );

    T::Cp2::set(core, rd, elem, core.get(rt));
}

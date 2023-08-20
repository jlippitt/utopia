use super::super::{Bus, Coprocessor0, Core, REGS};
use tracing::debug;

pub fn mfc0<T: Bus>(core: &mut Core<T>, _rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFC0 {}, {}", core.pc, REGS[rt], T::Cp0::REGS[rd]);
    let result = T::Cp0::get(core, rd);
    core.set(rt, result);
}

pub fn mtc0<T: Bus>(core: &mut Core<T>, _rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MTC0 {}, {}", core.pc, REGS[rt], T::Cp0::REGS[rd]);
    let result = core.get(rt);
    T::Cp0::set(core, rd, result);
}

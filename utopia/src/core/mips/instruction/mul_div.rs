use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn multu(core: &mut Core<impl Bus>, rs: usize, rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} MULTU {}, {}", core.pc, REGS[rs], REGS[rt]);
    let lhs = core.get(rs) as u64;
    let rhs = core.get(rt) as u64;
    core.hi_lo = lhs * rhs;
    debug!("  {} * {} = {}", lhs, rhs, core.hi_lo);
}

pub fn mflo(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFLO {}", core.pc, REGS[rd]);
    core.set(rd, core.hi_lo as u32);
}

pub fn mfhi(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFHI {}", core.pc, REGS[rd]);
    core.set(rd, (core.hi_lo >> 32) as u32);
}

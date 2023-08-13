use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn multu(core: &mut Core<impl Bus>, rs: usize, rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} MULTU {}, {}", core.pc, REGS[rs], REGS[rt]);
    let lhs = core.get(rs) as u64;
    let rhs = core.get(rt) as u64;
    let result = lhs * rhs;
    debug!("  {} * {} = {}", lhs, rhs, result);
    core.set_hi((result >> 32) as u32);
    core.set_lo(result as u32);
}

pub fn dmultu(core: &mut Core<impl Bus>, rs: usize, rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} DMULTU {}, {}", core.pc, REGS[rs], REGS[rt]);
    let lhs = core.get(rs) as u128;
    let rhs = core.get(rt) as u128;
    let result = lhs * rhs;
    debug!("  {} * {} = {}", lhs, rhs, result);
    core.setd_hi((result >> 64) as u64);
    core.setd_lo(result as u64);
}

pub fn mflo(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFLO {}", core.pc, REGS[rd]);
    core.setd(rd, core.lo);
}

pub fn mfhi(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFHI {}", core.pc, REGS[rd]);
    core.setd(rd, core.hi);
}

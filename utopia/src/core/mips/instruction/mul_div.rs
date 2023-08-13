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

pub fn mflo(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFLO {}", core.pc, REGS[rd]);
    core.set(rd, core.lo());
}

pub fn mfhi(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFHI {}", core.pc, REGS[rd]);
    core.set(rd, core.hi());
}

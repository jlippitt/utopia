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
    let lhs = core.getd(rs) as u128;
    let rhs = core.getd(rt) as u128;
    let result = lhs * rhs;
    core.setd_hi((result >> 64) as u64);
    core.setd_lo(result as u64);
}

pub fn divu(core: &mut Core<impl Bus>, rs: usize, rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} DIVU {}, {}", core.pc, REGS[rs], REGS[rt]);
    let lhs = core.get(rs);
    let rhs = core.get(rt);
    let quotient = lhs / rhs;
    let remainder = lhs % rhs;
    debug!("  {} * {} = {} ({})", lhs, rhs, quotient, remainder);
    core.set_hi(remainder);
    core.set_lo(quotient);
}

pub fn ddivu(core: &mut Core<impl Bus>, rs: usize, rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} DDIVU {}, {}", core.pc, REGS[rs], REGS[rt]);
    let lhs = core.getd(rs);
    let rhs = core.getd(rt);
    let quotient = lhs / rhs;
    let remainder = lhs % rhs;
    debug!("  {} * {} = {} ({})", lhs, rhs, quotient, remainder);
    core.setd_hi(remainder);
    core.setd_lo(quotient);
}

pub fn mflo(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFLO {}", core.pc, REGS[rd]);
    core.setd(rd, core.lo);
}

pub fn mfhi(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MFHI {}", core.pc, REGS[rd]);
    core.setd(rd, core.hi);
}

pub fn mtlo(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MTLO {}", core.pc, REGS[rd]);
    core.setd_lo(core.getd(rd));
}

pub fn mthi(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, rd: usize, _sa: u32) {
    debug!("{:08X} MTHI {}", core.pc, REGS[rd]);
    core.setd_hi(core.getd(rd));
}

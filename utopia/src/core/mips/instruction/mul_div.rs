use super::super::opcode::RType;
use super::super::{Bus, Core, GPR};
use tracing::trace;

pub fn mult(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} MULT {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let result = (core.getw(op.rs()) as i32 as i64) * (core.getw(op.rt()) as i32 as i64);

    core.setw_hi((result >> 32) as u32);
    core.setw_lo(result as u32);
}

pub fn multu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} MULTU {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let result = (core.getw(op.rs()) as u64) * (core.getw(op.rt()) as u64);

    core.setw_hi((result >> 32) as u32);
    core.setw_lo(result as u32);
}

pub fn dmult(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} DMULT {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let result = (core.getd(op.rs()) as i64 as i128) * (core.getd(op.rt()) as i64 as i128);

    core.setd_hi((result >> 64) as u64);
    core.setd_lo(result as u64);
}

pub fn dmultu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!(
        "{:08X} DMULTU {}, {}",
        core.pc(),
        GPR[op.rs()],
        GPR[op.rt()]
    );

    let result = (core.getd(op.rs()) as u128) * (core.getd(op.rt()) as u128);

    core.setd_hi((result >> 64) as u64);
    core.setd_lo(result as u64);
}

pub fn div(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} DIV {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let lhs = core.getw(op.rs()) as i32;
    let rhs = core.getw(op.rt()) as i32;

    if rhs != 0 {
        core.setw_hi(lhs.wrapping_rem(rhs) as u32);
        core.setw_lo(lhs.wrapping_div(rhs) as u32);
    } else {
        core.setw_hi(lhs as u32);
        core.setw_lo(if lhs < 0 { 1 } else { u32::MAX });
    }
}

pub fn divu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} DIVU {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let lhs = core.getw(op.rs());
    let rhs = core.getw(op.rt());

    if rhs != 0 {
        core.setw_hi(lhs % rhs);
        core.setw_lo(lhs / rhs);
    } else {
        core.setw_hi(lhs);
        core.setw_lo(u32::MAX);
    }
}

pub fn ddiv(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} DDIV {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let lhs = core.getd(op.rs()) as i64;
    let rhs = core.getd(op.rt()) as i64;

    if rhs != 0 {
        core.setd_hi(lhs.wrapping_rem(rhs) as u64);
        core.setd_lo(lhs.wrapping_div(rhs) as u64);
    } else {
        core.setd_hi(lhs as u64);
        core.setd_lo(if lhs < 0 { 1 } else { u64::MAX });
    }
}

pub fn ddivu(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);

    trace!("{:08X} DDIVU {}, {}", core.pc(), GPR[op.rs()], GPR[op.rt()]);

    let lhs = core.getd(op.rs());
    let rhs = core.getd(op.rt());

    if rhs != 0 {
        core.setd_hi(lhs % rhs);
        core.setd_lo(lhs / rhs);
    } else {
        core.setd_hi(lhs);
        core.setd_lo(u64::MAX);
    }
}

pub fn mfhi(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X}: MFHI {}", core.pc(), GPR[op.rd()]);
    core.setd(op.rd(), core.hi());
}

pub fn mthi(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X}: MTHI {}", core.pc(), GPR[op.rd()]);
    core.setd_hi(core.getd(op.rd()));
}

pub fn mflo(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X}: MFLO {}", core.pc(), GPR[op.rd()]);
    core.setd(op.rd(), core.lo());
}

pub fn mtlo(core: &mut Core<impl Bus>, word: u32) {
    let op = RType::from(word);
    trace!("{:08X}: MTLO {}", core.pc(), GPR[op.rd()]);
    core.setd_lo(core.getd(op.rd()));
}

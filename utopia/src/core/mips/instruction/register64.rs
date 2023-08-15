use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn dsll(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} DSLL {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = core.getd(rt) << sa;
    core.setd(rd, result);
}

pub fn dsll32(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} DSLL32 {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = core.getd(rt) << (sa + 32);
    core.setd(rd, result);
}

pub fn dsrl(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} DSRL {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = core.getd(rt) >> sa;
    core.setd(rd, result);
}

pub fn dsrl32(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} DSRL32 {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = core.getd(rt) >> (sa + 32);
    core.setd(rd, result);
}

pub fn dsra(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} DSRA {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = (core.getd(rt) as i64 >> sa) as u64;
    core.setd(rd, result);
}

pub fn dsra32(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} DSRA32 {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = (core.getd(rt) as i64 >> (sa + 32)) as u64;
    core.setd(rd, result);
}

pub fn dsllv(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} DSLLV {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );
    let result = core.getd(rt) << (core.getd(rs) & 63);
    core.setd(rd, result);
}

pub fn dsrlv(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} DSRLV {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );
    let result = core.getd(rt) >> (core.getd(rs) & 63);
    core.setd(rd, result);
}

pub fn dsrav(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} DSRAV {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );
    let result = (core.getd(rt) as i64 >> (core.getd(rs) & 63)) as u64;
    core.setd(rd, result);
}

pub fn and(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} AND {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.getd(rs) & core.getd(rt);
    core.setd(rd, result);
}

pub fn or(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} OR {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.getd(rs) | core.getd(rt);
    core.setd(rd, result);
}

pub fn xor(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} XOR {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.getd(rs) ^ core.getd(rt);
    core.setd(rd, result);
}

pub fn nor(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} NOR {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = !(core.getd(rs) | core.getd(rt));
    core.setd(rd, result);
}

pub fn slt(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SLT {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );
    let result = (core.getd(rs) as i64) < (core.getd(rt) as i64);
    core.setd(rd, result as u64);
}

pub fn sltu(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SLTU {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );
    let result = core.getd(rs) < core.getd(rt);
    core.setd(rd, result as u64);
}

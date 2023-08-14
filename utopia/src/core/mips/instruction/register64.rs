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

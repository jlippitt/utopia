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

use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn sll(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    if rt == 0 && rd == 0 && sa == 0 {
        debug!("{:08X} NOP", core.pc);
    } else {
        debug!("{:08X} SLL {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    }

    let result = core.get(rt) << sa;
    core.set(rd, result);
}

pub fn srl(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} SRL {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = core.get(rt) >> sa;
    core.set(rd, result);
}

pub fn addu(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} ADDU {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.get(rs).wrapping_add(core.get(rt));
    core.set(rd, result);
}

pub fn subu(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SUBU {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.get(rs).wrapping_sub(core.get(rt));
    core.set(rd, result);
}

pub fn or(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} OR {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.get(rs) | core.get(rt);
    core.set(rd, result);
}

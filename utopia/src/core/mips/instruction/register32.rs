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

pub fn sra(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    debug!("{:08X} SRA {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    let result = (core.getd(rt) as i64 >> sa) as u32;
    core.set(rd, result);
}

pub fn sllv(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SLLV {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );
    let result = core.get(rt) << (core.get(rs) & 31);
    core.set(rd, result);
}

pub fn srlv(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SRLV {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );
    let result = core.get(rt) >> (core.get(rs) & 31);
    core.set(rd, result);
}

pub fn srav(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SRAV {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );
    let result = (core.getd(rt) as i64 >> (core.get(rs) & 31)) as u32;
    core.set(rd, result);
}

pub fn add(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} ADD {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let (result, overflow) = (core.get(rs) as i32).overflowing_add(core.get(rt) as i32);

    if overflow {
        todo!("Overflow exceptions");
    }

    core.set(rd, result as u32);
}

pub fn addu(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} ADDU {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.get(rs).wrapping_add(core.get(rt));
    core.set(rd, result);
}

pub fn sub(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SUB {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let (result, overflow) = (core.get(rs) as i32).overflowing_sub(core.get(rt) as i32);

    if overflow {
        todo!("Overflow exceptions");
    }

    core.set(rd, result as u32);
}

pub fn subu(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} SUBU {}, {}, {}",
        core.pc, REGS[rd], REGS[rs], REGS[rt]
    );

    let result = core.get(rs).wrapping_sub(core.get(rt));
    core.set(rd, result);
}

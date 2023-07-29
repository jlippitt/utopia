use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn addi(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} ADDI {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    let ivalue = value as i16 as i32;
    let (result, overflow) = (core.get(rs) as i32).overflowing_add(ivalue);

    if overflow {
        todo!("Overflow exceptions");
    }

    core.set(rt, result as u32);
}

pub fn addiu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} ADDIU {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    let ivalue = value as i16 as i32 as u32;
    let result = core.get(rs).wrapping_add(ivalue);
    core.set(rt, result);
}

pub fn slti(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SLTI {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    let ivalue = value as i16 as i32;
    let result = (core.get(rs) as i32).wrapping_sub(ivalue);
    core.set(rt, (result < 0) as u32);
}

pub fn andi(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} ANDI {}, {}, 0x{:04X}",
        core.pc, REGS[rt], REGS[rs], value,
    );

    let result = core.get(rs) & value;
    core.set(rt, result);
}

pub fn ori(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} ORI {}, {}, 0x{:04X}",
        core.pc, REGS[rt], REGS[rs], value,
    );

    let result = core.get(rs) | value;
    core.set(rt, result);
}

pub fn xori(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} XORI {}, {}, 0x{:04X}",
        core.pc, REGS[rt], REGS[rs], value,
    );

    let result = core.get(rs) ^ value;
    core.set(rt, result);
}

pub fn sll(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize, sa: u32) {
    if rt == 0 && rd == 0 && sa == 0 {
        debug!("{:08X} NOP", core.pc);
    } else {
        debug!("{:08X} SLL {}, {}, {}", core.pc, REGS[rd], REGS[rt], sa);
    }

    let result = core.get(rt) << sa;
    core.set(rd, result);
}

pub fn or(core: &mut Core<impl Bus>, rs: usize, rt: usize, rd: usize, _sa: u32) {
    debug!(
        "{:08X} OR {}, {}, {}",
        core.pc, REGS[rd], REGS[rt], REGS[rs]
    );

    let result = core.get(rs) | core.get(rt);
    core.set(rd, result);
}

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

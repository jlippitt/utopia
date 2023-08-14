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
    let result = (core.get(rs) as i32) < ivalue;
    core.set(rt, result as u32);
}

pub fn sltiu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SLTIU {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    let ivalue = value as i16 as i32 as u32;
    let result = core.get(rs) < ivalue;
    core.set(rt, result as u32);
}

use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn daddi(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} DADDI {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    let ivalue = value as i16 as i64;
    let (result, overflow) = (core.getd(rs) as i64).overflowing_add(ivalue);

    if overflow {
        todo!("Overflow exceptions");
    }

    core.setd(rt, result as u64);
}

pub fn daddiu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} DADDIU {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    let ivalue = value as i16 as i64 as u64;
    let result = core.getd(rs).wrapping_add(ivalue);
    core.setd(rt, result);
}

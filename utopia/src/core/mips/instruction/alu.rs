use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn addiu(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} ADDIU {}, {}, 0x{:04X}",
        core.pc, REGS[rt], REGS[rs], value
    );

    let ivalue = value as i16 as i32 as u32;
    let result = core.get(rs) + ivalue;
    core.set(rt, result);
}

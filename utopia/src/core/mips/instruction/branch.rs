use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn bne(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} BNE {}, {}, {}",
        core.pc, REGS[rt], REGS[rs], value as i16
    );

    if core.get(rs) != core.get(rt) {
        debug!("  Branch taken");
        let ivalue = value as i16 as i32 as u32;
        core.pc = core.pc.wrapping_add(ivalue);
    } else {
        debug!("  Branch not taken");
    }
}

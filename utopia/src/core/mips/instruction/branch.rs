use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn bne(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    let offset = (value as i16 as i32) << 2;

    debug!("{:08X} BNE {}, {}, {}", core.pc, REGS[rt], REGS[rs], offset);

    if core.get(rs) != core.get(rt) {
        debug!("  Branch taken");
        core.next[1] = core.next[0].wrapping_add(offset as u32);
    } else {
        debug!("  Branch not taken");
    }
}

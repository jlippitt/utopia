use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn sw(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} SW {}, {}({})",
        core.pc, REGS[rt], value as i16, REGS[rs]
    );

    let ivalue = value as i16 as i32 as u32;
    let address = core.get(rs).wrapping_add(ivalue);
    core.write_word(address, core.get(rt));
}

use super::super::{Bus, Core};
use tracing::debug;

pub fn branch<const LINK: bool>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let offset = ((word as i32) << 8) >> 6;

    debug!("{:08X} B{} {:+}", pc, if LINK { "L" } else { "" }, offset);

    if LINK {
        core.regs[14] = core.pc;
    }

    core.pc = core.pc.wrapping_add(4).wrapping_add(offset as u32);
}

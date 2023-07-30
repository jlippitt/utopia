use super::{Bus, Core};
use tracing::debug;

pub fn branch<const LINK: bool>(core: &mut Core<impl Bus>, word: u32) {
    let offset = ((word as i32) << 8) >> 6;

    debug!(
        "{:08X} B{} {:+}",
        core.pc,
        if LINK { "L" } else { "" },
        offset
    );

    if LINK {
        core.regs[14] = core.pc.wrapping_add(4);
    }

    core.pc = core.pc.wrapping_add(8).wrapping_add(offset as u32);
}

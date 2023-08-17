use super::super::{Bus, Core};
use tracing::debug;

pub fn branch<const C_VALUE: bool, const LIKELY: bool>(core: &mut Core<impl Bus>, word: u32) {
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    debug!(
        "{:08X} BC1{}{} {:+}",
        core.pc,
        if C_VALUE { "T" } else { "F" },
        if LIKELY { "L" } else { "" },
        offset
    );

    if core.cp1.ctrl.c == C_VALUE {
        debug!("  Branch taken");
        core.jump_delayed(core.next[0].wrapping_add(offset as u32));
    } else {
        debug!("  Branch not taken");

        if LIKELY {
            // Skip the delay slot
            core.next[0] = core.next[1];
            core.next[1] = core.next[1].wrapping_add(4);
        }
    }
}

use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn cache(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    debug!(
        "{:08X} CACHE 0x{:X}, {}({})",
        core.pc, rt, value as i16, REGS[rs],
    );

    // TODO: Caching
}

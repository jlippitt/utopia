use super::super::{Bus, Core, REGS};
use tracing::trace;

pub fn cache(core: &mut Core<impl Bus>, rs: usize, rt: usize, value: u32) {
    trace!(
        "{:08X} CACHE 0x{:X}, {}({})",
        core.pc,
        rt,
        value as i16,
        REGS[rs],
    );

    // TODO: Caching
}

pub fn sync(core: &mut Core<impl Bus>, _rs: usize, _rt: usize, _rd: usize, _sa: u32) {
    trace!("{:08X} SYNC", core.pc);

    // This is a NOP on the VR4300
}

use super::super::{Bus, Core};
use tracing::debug;

pub fn nop(core: &mut Core<impl Bus>) {
    debug!("NOP");
    core.read(core.pc);
}

pub fn csl(core: &mut Core<impl Bus>) {
    debug!("CSL");
    core.read(core.pc);
    // TODO: Clock speed
}

pub fn csh(core: &mut Core<impl Bus>) {
    debug!("CSH");
    core.read(core.pc);
    // TODO: Clock speed
}

use super::super::{Bus, Core};
use tracing::debug;

pub fn pha(core: &mut Core<impl Bus>) {
    debug!("PHA");
    core.read(core.pc);
    core.poll();
    core.push(core.a);
}
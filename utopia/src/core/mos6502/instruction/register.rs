use super::super::{Bus, Core};
use tracing::debug;

pub fn txs(core: &mut Core<impl Bus>) {
    debug!("TXS");
    core.poll();
    core.read(core.pc);
    core.s = core.x;
}

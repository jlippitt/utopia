use super::super::{Bus, Core};
use tracing::trace;

pub fn di(_core: &mut Core<impl Bus>) {
    trace!("DI");
    // TODO: Interrupt handling
}

pub fn im(_core: &mut Core<impl Bus>, mode: u8) {
    trace!("IM {}", mode);
    // TODO: Interrupt handling
}

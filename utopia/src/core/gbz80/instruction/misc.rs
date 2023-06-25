use super::super::{Bus, Core};
use tracing::debug;

pub fn nop(_core: &mut Core<impl Bus>) {
    debug!("NOP");
}

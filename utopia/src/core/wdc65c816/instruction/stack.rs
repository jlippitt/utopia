use super::super::{Bus, Core};
use tracing::debug;

pub fn php<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PHP");
    core.idle();
    core.poll();
    core.push::<E>(core.flags_to_u8::<E>(true));
}

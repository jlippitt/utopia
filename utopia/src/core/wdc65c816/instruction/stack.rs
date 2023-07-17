use super::super::{Bus, Core};
use tracing::debug;

pub fn php<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PHP");
    core.idle();
    core.poll();
    core.push::<E>(core.flags_to_u8::<E>(true));
}

pub fn pha<const E: bool, const M: bool>(core: &mut Core<impl Bus>) {
    debug!("PHA.{}", super::size(M));
    core.idle();

    if !M {
        core.push::<E>((core.a >> 8) as u8);
    }

    core.poll();
    core.push::<E>(core.a as u8);
}

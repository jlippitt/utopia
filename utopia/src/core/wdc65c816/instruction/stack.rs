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

pub fn pla<const E: bool, const M: bool>(core: &mut Core<impl Bus>) {
    debug!("PLA.{}", super::size(M));
    core.idle();
    core.idle();

    if M {
        let value = core.pull::<E>();
        core.a = (core.a & 0xff00) | (value as u16);
        core.set_nz8(value);
    } else {
        let low = core.pull::<E>();
        let high = core.pull::<E>();
        core.a = u16::from_le_bytes([low, high]);
        core.set_nz16(core.a);
    }
}

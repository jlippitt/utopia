use super::super::{Bus, Core};
use tracing::trace;

pub fn outi(core: &mut Core<impl Bus>) {
    trace!("OUTI");
    out::<true>(core);
}

pub fn otir(core: &mut Core<impl Bus>) {
    trace!("OTIR");
    repeat(core, out::<true>);
}

pub fn outd(core: &mut Core<impl Bus>) {
    trace!("OUTI");
    out::<false>(core);
}

pub fn otdr(core: &mut Core<impl Bus>) {
    trace!("OTIR");
    repeat(core, out::<false>);
}

fn repeat<T: Bus>(core: &mut Core<T>, cb: impl Fn(&mut Core<T>) -> bool) {
    if cb(core) {
        core.idle(5);
        core.pc = core.pc.wrapping_sub(2);
    }
}

fn out<const INC: bool>(core: &mut Core<impl Bus>) -> bool {
    core.idle(1);

    let counter = ((core.bc >> 8) as u8).wrapping_sub(1);
    core.bc = (core.bc & 0xff) | ((counter as u16) << 8);
    core.set_sz(counter);

    let value = core.read(core.hl);
    core.write_port(core.bc, value);

    if INC {
        core.hl = core.hl.wrapping_add(1);
    } else {
        core.hl = core.hl.wrapping_sub(1);
    }

    core.flags.n = (value & 0x80) != 0;
    core.flags.h = value.checked_add(core.hl as u8).is_none();
    core.flags.c = core.flags.h;
    // TODO: Parity flag

    counter != 0
}

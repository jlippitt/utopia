use super::super::{Bus, Core};
use tracing::trace;

pub fn ldi(core: &mut Core<impl Bus>) {
    trace!("LDI");
    ld::<true>(core);
}

pub fn ldir(core: &mut Core<impl Bus>) {
    trace!("LDIR");
    repeat(core, ld::<true>);
}

pub fn ldd(core: &mut Core<impl Bus>) {
    trace!("LDD");
    ld::<false>(core);
}

pub fn lddr(core: &mut Core<impl Bus>) {
    trace!("LLDR");
    repeat(core, ld::<false>);
}

pub fn outi(core: &mut Core<impl Bus>) {
    trace!("OUTI");
    out::<true>(core);
}

pub fn otir(core: &mut Core<impl Bus>) {
    trace!("OTIR");
    repeat(core, out::<true>);
}

pub fn outd(core: &mut Core<impl Bus>) {
    trace!("OUTD");
    out::<false>(core);
}

pub fn otdr(core: &mut Core<impl Bus>) {
    trace!("OTDR");
    repeat(core, out::<false>);
}

fn repeat<T: Bus>(core: &mut Core<T>, cb: impl Fn(&mut Core<T>) -> bool) {
    if cb(core) {
        core.idle(5);
        core.pc = core.pc.wrapping_sub(2);
    }
}

fn ld<const INC: bool>(core: &mut Core<impl Bus>) -> bool {
    core.bc = core.bc.wrapping_sub(1);

    let value = core.read(core.hl);
    core.write(core.de, value);
    core.idle(2);

    if INC {
        core.de = core.de.wrapping_add(1);
        core.hl = core.hl.wrapping_add(1);
    } else {
        core.de = core.de.wrapping_sub(1);
        core.hl = core.hl.wrapping_sub(1);
    }

    core.flags.x = value.wrapping_add(core.a);
    core.flags.y = core.flags.x << 4;
    core.flags.h = false;
    core.flags.pv = core.bc != 0;

    core.bc != 0
}

fn out<const INC: bool>(core: &mut Core<impl Bus>) -> bool {
    core.idle(1);

    let counter = ((core.bc >> 8) as u8).wrapping_sub(1);
    core.bc = (core.bc & 0xff) | ((counter as u16) << 8);

    let value = core.read(core.hl);
    core.write_port(core.bc, value);

    if INC {
        core.hl = core.hl.wrapping_add(1);
    } else {
        core.hl = core.hl.wrapping_sub(1);
    }

    core.set_sz(counter);
    core.flags.n = (value & 0x80) != 0;
    core.flags.h = value.checked_add(core.hl as u8).is_none();
    core.flags.c = core.flags.h;
    // TODO: Parity flag

    counter != 0
}

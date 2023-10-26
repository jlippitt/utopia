use super::super::{Bus, Core, Mode, EMULATION_STACK_PAGE};
use std::mem;
use tracing::{trace, warn};

pub fn nop(core: &mut Core<impl Bus>) {
    trace!("NOP");
    core.poll();
    core.idle();
}

pub fn wdm(core: &mut Core<impl Bus>) {
    trace!("WDM #const");
    core.poll();
    core.next_byte();
}

pub fn xba(core: &mut Core<impl Bus>) {
    trace!("XBA");
    core.poll();
    core.idle();
    core.idle();
    core.a = core.a.swap_bytes();
    core.set_nz8(core.a as u8);
}

pub fn xce(core: &mut Core<impl Bus>) {
    trace!("XCE");
    core.poll();
    core.idle();

    let emulation_mode = core.flags.c;
    core.flags.c = (core.mode as u8 & 0x04) != 0;

    if emulation_mode {
        core.mode = Mode::Emulation;
        core.x &= 0xff;
        core.y &= 0xff;
        core.s = EMULATION_STACK_PAGE | (core.s & 0xff);
    } else {
        core.mode = unsafe { mem::transmute(core.mode as u8 & !0x04) };
    }
}

pub fn wai(core: &mut Core<impl Bus>) {
    trace!("WAI");
    core.idle();
    core.waiting = true;
}

pub fn stp(core: &mut Core<impl Bus>) {
    trace!("STP");
    core.idle();
    core.stopped = true;
    warn!("Processor stopped");
}

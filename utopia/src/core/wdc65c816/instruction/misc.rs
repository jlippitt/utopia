use super::super::{Bus, Core, Mode, EMULATION_STACK_PAGE};
use tracing::debug;

pub fn xba(core: &mut Core<impl Bus>) {
    debug!("XBA");
    core.poll();
    core.idle();
    core.idle();
    core.a = core.a.swap_bytes();
    core.set_nz8(core.a as u8);
}

pub fn xce(core: &mut Core<impl Bus>) {
    debug!("XCE");
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
        core.mode = Mode::Native11;
    }
}

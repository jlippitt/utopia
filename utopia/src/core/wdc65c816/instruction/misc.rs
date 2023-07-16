use super::super::{Bus, Core, Mode, EMULATION_STACK_PAGE};
use tracing::debug;

pub fn rep<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("REP #const");
    let value = core.next_byte();
    core.poll();
    core.idle();
    core.flags_from_u8::<E>(core.flags_to_u8::<E>(false) & !value);
}

pub fn sep<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("REP #const");
    let value = core.next_byte();
    core.poll();
    core.idle();
    core.flags_from_u8::<E>(core.flags_to_u8::<E>(false) | value);
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

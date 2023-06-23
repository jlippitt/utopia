use super::super::{Core, Bus, Condition};
use tracing::debug;

pub fn jr(core: &mut Core<impl Bus>) {
    debug!("JR PC+i8");
    let offset = core.next_byte() as i8;
    core.idle();
    core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
}

pub fn jr_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    debug!("JR {}, PC+i8", Cond::NAME);
    let offset = core.next_byte() as i8;

    if Cond::test(&core.flags) {
        core.idle();
        core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
    }
}
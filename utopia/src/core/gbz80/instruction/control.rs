use super::super::{Bus, Condition, Core};
use tracing::debug;

pub fn jp(core: &mut Core<impl Bus>) {
    debug!("JP u16");
    core.pc = core.next_word();
    core.idle();
}

pub fn jp_hl(core: &mut Core<impl Bus>) {
    debug!("JP HL");
    core.pc = core.hl;
}

pub fn jp_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    debug!("JP {}, u16", Cond::NAME);
    let target = core.next_word();

    if Cond::test(&core.flags) {
        debug!("Branch taken");
        core.idle();
        core.pc = target;
    } else {
        debug!("Branch not taken");
    }
}

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
        debug!("Branch taken");
        core.idle();
        core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
    } else {
        debug!("Branch not taken");
    }
}

pub fn call(core: &mut Core<impl Bus>) {
    debug!("CALL u16");
    let target = core.next_word();
    core.idle();
    core.push(core.pc);
    core.pc = target;
}

pub fn call_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    debug!("CALL {}, u16", Cond::NAME);
    let target = core.next_word();

    if Cond::test(&core.flags) {
        debug!("Branch taken");
        core.idle();
        core.push(core.pc);
        core.pc = target;
    } else {
        debug!("Branch not taken");
    }
}

pub fn ret(core: &mut Core<impl Bus>) {
    debug!("RET");
    core.pc = core.pop();
    core.idle();
}

pub fn ret_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    debug!("RET {}", Cond::NAME);
    core.idle();

    if Cond::test(&core.flags) {
        debug!("Branch taken");
        core.pc = core.pop();
        core.idle();
    } else {
        debug!("Branch not taken");
    }
}

use super::super::condition::Condition;
use super::super::{Bus, Core};
use tracing::trace;

pub fn jp(core: &mut Core<impl Bus>) {
    trace!("JP u16");
    core.pc = core.next_word();
    core.idle();
}

pub fn jp_hl(core: &mut Core<impl Bus>) {
    trace!("JP HL");
    core.pc = core.hl;
}

pub fn jp_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("JP {}, u16", Cond::NAME);
    let target = core.next_word();

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.idle();
        core.pc = target;
    } else {
        trace!("Branch not taken");
    }
}

pub fn jr(core: &mut Core<impl Bus>) {
    trace!("JR PC+i8");
    let offset = core.next_byte() as i8;
    core.idle();
    core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
}

pub fn jr_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("JR {}, PC+i8", Cond::NAME);
    let offset = core.next_byte() as i8;

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.idle();
        core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
    } else {
        trace!("Branch not taken");
    }
}

pub fn call(core: &mut Core<impl Bus>) {
    trace!("CALL u16");
    let target = core.next_word();
    core.idle();
    core.push(core.pc);
    core.pc = target;
}

pub fn call_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("CALL {}, u16", Cond::NAME);
    let target = core.next_word();

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.idle();
        core.push(core.pc);
        core.pc = target;
    } else {
        trace!("Branch not taken");
    }
}

pub fn ret(core: &mut Core<impl Bus>) {
    trace!("RET");
    core.pc = core.pop();
    core.idle();
}

pub fn ret_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("RET {}", Cond::NAME);
    core.idle();

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.pc = core.pop();
        core.idle();
    } else {
        trace!("Branch not taken");
    }
}

pub fn rst(core: &mut Core<impl Bus>, target: u8) {
    trace!("RST ${:02X}", target);
    core.idle();
    core.push(core.pc);
    core.pc = target as u16;
}

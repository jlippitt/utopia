use super::super::address_mode::{ReadAddress, WriteAddress, B};
use super::super::condition::Condition;
use super::super::{Bus, Core};
use tracing::trace;

pub fn jp(core: &mut Core<impl Bus>) {
    trace!("JP u16");
    core.pc = core.next_word();
    core.idle(1);
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
        core.idle(1);
        core.pc = target;
    } else {
        trace!("Branch not taken");
    }
}

pub fn jr(core: &mut Core<impl Bus>) {
    trace!("JR PC+i8");
    let offset = core.next_byte() as i8;
    core.idle(5);
    core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
}

pub fn jr_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("JR {}, PC+i8", Cond::NAME);
    let offset = core.next_byte() as i8;

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.idle(5);
        core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
    } else {
        trace!("Branch not taken");
    }
}

pub fn djnz(core: &mut Core<impl Bus>) {
    trace!("DJNZ PC+i8");

    core.idle(1);
    let counter = B::read(core).wrapping_sub(1);
    B::write(core, counter);

    let offset = core.next_byte() as i8;

    if counter != 0 {
        trace!("Branch taken");
        core.idle(5);
        core.pc = (core.pc as i16).wrapping_add(offset as i16) as u16;
    } else {
        trace!("Branch not taken");
    }
}

pub fn call(core: &mut Core<impl Bus>) {
    trace!("CALL u16");
    let target = core.next_word();
    core.idle(1);
    core.push(core.pc);
    core.pc = target;
}

pub fn call_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("CALL {}, u16", Cond::NAME);
    let target = core.next_word();

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.idle(1);
        core.push(core.pc);
        core.pc = target;
    } else {
        trace!("Branch not taken");
    }
}

pub fn ret(core: &mut Core<impl Bus>) {
    trace!("RET");
    core.pc = core.pop();
    core.idle(1);
}

pub fn ret_conditional<Cond: Condition>(core: &mut Core<impl Bus>) {
    trace!("RET {}", Cond::NAME);
    core.idle(1);

    if Cond::test(&core.flags) {
        trace!("Branch taken");
        core.pc = core.pop();
        core.idle(1);
    } else {
        trace!("Branch not taken");
    }
}

pub fn rst(core: &mut Core<impl Bus>, target: u8) {
    trace!("RST ${:02X}", target);
    core.idle(1);
    core.push(core.pc);
    core.pc = target as u16;
}

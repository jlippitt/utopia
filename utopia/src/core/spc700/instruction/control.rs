use super::super::address_mode::ReadAddress;
use super::super::operator::BranchOperator;
use super::super::{Bus, Core};
use tracing::debug;

fn execute_branch(core: &mut Core<impl Bus>, condition: bool) {
    let offset = core.next_byte();

    if condition {
        debug!("Branch taken");
        core.pc = core.pc.wrapping_add(offset as i8 as i16 as u16);
        core.idle();
        core.idle();
    } else {
        debug!("Branch not taken");
    }
}

pub fn jmp(core: &mut Core<impl Bus>) {
    debug!("JMP !a");
    core.pc = core.next_word();
}

pub fn jmp_x_indirect(core: &mut Core<impl Bus>) {
    debug!("JMP [!a+X]");
    let low_address = core.next_word().wrapping_add(core.x as u16);
    core.idle();
    let low = core.read(low_address);
    let high_address = low_address.wrapping_add(1);
    let high = core.read(high_address);
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn call(core: &mut Core<impl Bus>) {
    debug!("CALL !a");
    let target = core.next_word();
    core.idle();
    core.push((core.pc >> 8) as u8);
    core.push(core.pc as u8);
    core.pc = target;
    core.idle();
    core.idle();
}

pub fn ret(core: &mut Core<impl Bus>) {
    debug!("RET");
    core.read(core.pc);
    core.idle();
    let low = core.pop();
    let high = core.pop();
    core.pc = u16::from_le_bytes([low, high]);
}

pub fn branch<Op: BranchOperator>(core: &mut Core<impl Bus>) {
    debug!("{} r", Op::NAME);
    execute_branch(core, Op::apply(&core.flags));
}

pub fn cbne<Addr: ReadAddress>(core: &mut Core<impl Bus>) {
    debug!("CBNE {}, r", Addr::NAME);
    let value = Addr::read(core);
    core.idle();
    execute_branch(core, core.a != value);
}

pub fn dbnz_y(core: &mut Core<impl Bus>) {
    debug!("DBNZ Y, r");
    core.read(core.pc);
    core.idle();
    core.y = core.y.wrapping_sub(1);
    execute_branch(core, core.y != 0);
}

pub fn dbnz_direct(core: &mut Core<impl Bus>) {
    debug!("DBNZ d, r");
    let address = core.next_byte();
    let value = core.read_direct(address);
    let result = value.wrapping_sub(1);
    core.write_direct(address, result);
    execute_branch(core, result != 0);
}

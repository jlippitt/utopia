use super::super::operator::BranchOperator;
use super::super::{Bus, Core};
use tracing::debug;

pub fn branch<Op: BranchOperator>(core: &mut Core<impl Bus>) {
    debug!("{} r", Op::NAME);
    let offset = core.next_byte();

    if Op::apply(&core.flags) {
        debug!("Branch taken");
        core.pc = core.pc.wrapping_add(offset as i8 as i16 as u16);
        core.idle();
        core.idle();
    } else {
        debug!("Branch not taken");
    }
}

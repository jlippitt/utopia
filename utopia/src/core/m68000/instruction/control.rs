use super::condition::Condition;
use super::{Bus, Core};
use tracing::trace;

pub fn bra(core: &mut Core<impl Bus>, word: u16) {
    trace!("BRA label");
    branch(core, word);
}

pub fn bcc<T: Condition>(core: &mut Core<impl Bus>, word: u16) {
    trace!("B{} label", T::NAME);

    if T::apply(&core.flags) {
        trace!("  Branch taken");
        branch(core, word);
    } else {
        trace!("  Branch not taken");
    }
}

fn branch(core: &mut Core<impl Bus>, word: u16) {
    let pc = core.pc;
    let mut displacement = (word & 0xff) as i8 as i16;

    if displacement == 0 {
        displacement = core.next::<u16>() as i16;
    }

    core.pc = pc.wrapping_add(displacement as u32);
}

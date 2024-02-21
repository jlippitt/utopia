use super::{AddressMode, Bus, Condition, Core};
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

pub fn scc_dbcc<T: Condition>(core: &mut Core<impl Bus>, word: u16) {
    let operand = AddressMode::from(word);

    if operand.is_areg() {
        let index = operand.reg();
        trace!("DB{} D{}, label", T::NAME, index);

        if !T::apply(&core.flags) {
            let value: u16 = core.dreg(index);
            let result = value.wrapping_sub(1);
            core.set_dreg(index, result);

            if result != u16::MAX {
                trace!("  Branch taken");
                let pc = core.pc;
                let displacement = core.next::<u16>() as i16;
                core.pc = pc.wrapping_add(displacement as u32);
                return;
            }
        }

        trace!("  Branch not taken");
        core.pc = core.pc.wrapping_add(2);
    } else {
        todo!("Scc");
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

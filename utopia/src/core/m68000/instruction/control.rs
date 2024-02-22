use super::{AddressMode, Bus, Condition, Core};
use tracing::trace;

pub fn bra(core: &mut Core<impl Bus>, word: u16) {
    trace!("BRA label");
    core.pc = calc_target(core, word);
}

pub fn bsr(core: &mut Core<impl Bus>, word: u16) {
    trace!("BSR label");
    let target = calc_target(core, word);
    let sp = core.areg::<u32>(7).wrapping_sub(4);
    core.set_areg(7, sp);
    core.write(sp, core.pc);
    core.pc = target;
}

pub fn bcc<T: Condition>(core: &mut Core<impl Bus>, word: u16) {
    trace!("B{} label", T::NAME);

    if T::apply(&core.flags) {
        trace!("  Branch taken");
        core.pc = calc_target(core, word);
    } else {
        trace!("  Branch not taken");

        // Skip the 'displacement' word if present
        if (word & 0xff) == 0 {
            core.pc = core.pc.wrapping_add(2);
        }
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

pub fn rts(core: &mut Core<impl Bus>) {
    trace!("RTS");
    let sp: u32 = core.areg(7);
    core.pc = core.read(sp);
    core.set_areg(7, sp.wrapping_add(4));
}

fn calc_target(core: &mut Core<impl Bus>, word: u16) -> u32 {
    let pc = core.pc;
    let mut displacement = (word & 0xff) as i8 as i16;

    if displacement == 0 {
        displacement = core.next::<u16>() as i16;
    }

    pc.wrapping_add(displacement as u32)
}

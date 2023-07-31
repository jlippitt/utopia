use super::{Bus, Core};

pub fn dispatch(core: &mut Core<impl Bus>) {
    assert!((core.pc & 3) == 0);

    let pc = core.pc;
    let word = core.bus.read::<u16>(core.pc);
    core.pc = core.pc.wrapping_add(2);

    match word >> 8 {
        opcode => todo!("Thumb Opcode {0:02X} [{0:08b}] (PC: {1:08X})", opcode, pc),
    }
}

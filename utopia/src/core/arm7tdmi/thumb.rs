use super::operator as op;
use super::{Bus, Core};
use process::*;

mod process;

pub fn dispatch(core: &mut Core<impl Bus>) {
    assert!((core.pc & 1) == 0);

    let pc = core.pc;
    let word = core.bus.read::<u16>(core.pc);
    core.pc = core.pc.wrapping_add(2);

    match word >> 8 {
        0x20..=0x27 => move_immediate(core, pc, word),
        0x28..=0x2f => compare_immediate(core, pc, word),
        0x30..=0x37 => binary_immediate::<op::Add>(core, pc, word),
        //0x38..=0x3f => binary_immediate::<op::Sub>(core, pc, word),
        opcode => todo!("Thumb Opcode {0:02X} [{0:08b}] (PC: {1:08X})", opcode, pc),
    }
}

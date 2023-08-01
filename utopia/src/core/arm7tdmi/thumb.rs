use super::operator as op;
use super::{Bus, Core};
use process::*;
use transfer::*;

mod process;
mod transfer;

pub fn dispatch(core: &mut Core<impl Bus>) {
    assert!((core.pc & 1) == 0);

    let pc = core.pc;
    let word = core.bus.read::<u16>(core.pc);
    core.pc = core.pc.wrapping_add(2);

    match word >> 8 {
        0x18 | 0x19 => binary_register_3op::<op::Add>(core, pc, word),
        //0x1a | 0x1b => binary_register_3op::<op::Sub, false>(core, pc, word),
        0x1c | 0x1d => binary_immediate_3op::<op::Add>(core, pc, word),
        //0x1e | 0x1f => binary__immediate_3op::<op::Sub>(core, pc, word),
        0x20..=0x27 => move_immediate(core, pc, word),
        0x28..=0x2f => compare_immediate(core, pc, word),
        0x30..=0x37 => binary_immediate::<op::Add>(core, pc, word),
        //0x38..=0x3f => binary_immediate::<op::Sub>(core, pc, word),
        0x48..=0x4f => ldr_pc_relative(core, pc, word),
        0x50 | 0x51 => str_register::<false>(core, pc, word),
        0x54 | 0x55 => str_register::<true>(core, pc, word),
        0x58 | 0x59 => ldr_register::<false>(core, pc, word),
        0x5c | 0x5d => ldr_register::<true>(core, pc, word),
        opcode => todo!("Thumb Opcode {0:02X} [{0:08b}] (PC: {1:08X})", opcode, pc),
    }
}

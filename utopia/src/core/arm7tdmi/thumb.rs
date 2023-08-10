use super::operator as op;
use super::{Bus, Core};
use control::*;
use process::*;
use transfer::*;

mod control;
mod process;
mod transfer;

pub fn dispatch(core: &mut Core<impl Bus>) {
    assert!((core.pc & 1) == 0);

    let pc = core.pc;
    let word = core.bus.read::<u16>(core.pc);
    core.pc = core.pc.wrapping_add(2);

    match word >> 8 {
        0x00..=0x07 => move_shifted::<op::Lsl>(core, pc, word),
        0x08..=0x0f => move_shifted::<op::Lsr>(core, pc, word),
        //0x10..=0x17 => move_shifted::<op::Asr>(core, pc, word),
        0x18 | 0x19 => binary_register_3op::<op::Add>(core, pc, word),
        0x1a | 0x1b => binary_register_3op::<op::Sub>(core, pc, word),
        0x1c | 0x1d => binary_immediate_3op::<op::Add>(core, pc, word),
        0x1e | 0x1f => binary_immediate_3op::<op::Sub>(core, pc, word),
        0x20..=0x27 => move_immediate(core, pc, word),
        0x28..=0x2f => compare_immediate(core, pc, word),
        0x30..=0x37 => binary_immediate::<op::Add>(core, pc, word),
        0x38..=0x3f => binary_immediate::<op::Sub>(core, pc, word),
        0x40..=0x43 => alu_operation(core, pc, word),
        //0x44 => add_high(core, pc, word),
        //0x45 => cmp_high(core, pc, word),
        0x46 => mov_high(core, pc, word),
        0x47 => bx(core, pc, word),
        0x48..=0x4f => ldr_pc_relative(core, pc, word),
        0x50 | 0x51 => str_register::<false>(core, pc, word),
        0x54 | 0x55 => str_register::<true>(core, pc, word),
        0x58 | 0x59 => ldr_register::<false>(core, pc, word),
        0x5c | 0x5d => ldr_register::<true>(core, pc, word),
        0x80..=0x87 => str_halfword(core, pc, word),
        0x88..=0x8f => ldr_halfword(core, pc, word),
        0x90..=0x97 => str_sp_relative(core, pc, word),
        0x98..=0x9f => ldr_sp_relative(core, pc, word),
        0xb0 => add_sp_immediate(core, pc, word),
        0xb4 => push::<false>(core, pc, word),
        0xb5 => push::<true>(core, pc, word),
        0xbc => pop::<false>(core, pc, word),
        0xbd => pop::<true>(core, pc, word),
        0xd0..=0xde => branch_conditional(core, pc, word),
        0xe0..=0xe7 => branch_unconditional(core, pc, word),
        0xf0..=0xf7 => branch_and_link::<false>(core, pc, word),
        0xf8..=0xff => branch_and_link::<true>(core, pc, word),
        opcode => todo!("Thumb Opcode {0:02X} [{0:08b}] (PC: {1:08X})", opcode, pc),
    }
}

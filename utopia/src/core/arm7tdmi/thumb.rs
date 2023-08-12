use super::operator as op;
use super::{Bus, Core};
use block::*;
use control::*;
use process::*;
use transfer::*;

mod block;
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
        0x10..=0x17 => move_shifted::<op::Asr>(core, pc, word),

        0x18 | 0x19 => alu_register_3op::<op::Add>(core, pc, word),
        0x1a | 0x1b => alu_register_3op::<op::Sub>(core, pc, word),

        0x1c | 0x1d => alu_immediate_3op::<op::Add>(core, pc, word),
        0x1e | 0x1f => alu_immediate_3op::<op::Sub>(core, pc, word),

        0x20..=0x27 => alu_immediate_2op::<op::Mov>(core, pc, word),
        0x28..=0x2f => alu_immediate_2op::<op::Cmp>(core, pc, word),
        0x30..=0x37 => alu_immediate_2op::<op::Add>(core, pc, word),
        0x38..=0x3f => alu_immediate_2op::<op::Sub>(core, pc, word),

        0x40..=0x43 => alu_register_2op(core, pc, word),

        0x44 => alu_register_high::<op::Add>(core, pc, word),
        0x45 => alu_register_high::<op::Cmp>(core, pc, word),
        0x46 => alu_register_high::<op::Mov>(core, pc, word),

        0x47 => bx(core, pc, word),

        0x48..=0x4f => ldr_pc_relative(core, pc, word),

        0x50 | 0x51 => mem_register::<op::Str, 2>(core, pc, word),
        0x52 | 0x53 => mem_register::<op::Str, 1>(core, pc, word),
        0x54 | 0x55 => mem_register::<op::Str, 0>(core, pc, word),
        0x56 | 0x57 => mem_register::<op::Lds, 0>(core, pc, word),
        0x58 | 0x59 => mem_register::<op::Ldr, 2>(core, pc, word),
        0x5a | 0x5b => mem_register::<op::Ldr, 1>(core, pc, word),
        0x5c | 0x5d => mem_register::<op::Ldr, 0>(core, pc, word),
        0x5e | 0x5f => mem_register::<op::Lds, 1>(core, pc, word),

        0x60..=0x67 => mem_immediate::<op::Str, 2>(core, pc, word),
        0x68..=0x6f => mem_immediate::<op::Ldr, 2>(core, pc, word),

        0x70..=0x77 => mem_immediate::<op::Str, 0>(core, pc, word),
        0x78..=0x7f => mem_immediate::<op::Ldr, 0>(core, pc, word),

        0x80..=0x87 => mem_immediate::<op::Str, 1>(core, pc, word),
        0x88..=0x8f => mem_immediate::<op::Ldr, 1>(core, pc, word),

        0x90..=0x97 => str_sp_relative(core, pc, word),
        0x98..=0x9f => ldr_sp_relative(core, pc, word),

        0xa0..=0xa7 => load_address::<false>(core, pc, word),
        0xa8..=0xaf => load_address::<true>(core, pc, word),

        0xb0 => add_sp_immediate(core, pc, word),

        0xb4 => push::<false>(core, pc, word),
        0xb5 => push::<true>(core, pc, word),
        0xbc => pop::<false>(core, pc, word),
        0xbd => pop::<true>(core, pc, word),

        0xc0..=0xc7 => stmia(core, pc, word),
        0xc8..=0xcf => ldmia(core, pc, word),

        0xd0..=0xde => branch_conditional(core, pc, word),

        0xdf => swi(core, pc, word),

        0xe0..=0xe7 => branch_unconditional(core, pc, word),

        0xf0..=0xf7 => branch_and_link::<false>(core, pc, word),
        0xf8..=0xff => branch_and_link::<true>(core, pc, word),

        opcode => todo!("Thumb Opcode {0:02X} [{0:08b}] (PC: {1:08X})", opcode, pc),
    }
}

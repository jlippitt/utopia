use super::condition::Condition;
use super::operator::{self as op, ShiftOperator};
use super::{Bus, Core};
use block::*;
use control::*;
use num_traits::FromPrimitive;
use process::*;
use tracing::debug;
use transfer::*;

mod block;
mod control;
mod process;
mod transfer;

const SHIFT: [&str; 4] = ["LSL", "LSR", "ASR", "ROR"];

pub fn dispatch(core: &mut Core<impl Bus>) {
    assert!((core.pc & 3) == 0);

    let pc = core.pc;
    let word = core.bus.read::<u32>(core.pc);
    core.pc = core.pc.wrapping_add(4);

    let condition = Condition::from_u32(word >> 28).unwrap();

    if !condition.apply(core) {
        debug!("{:08X}: ({}: Skipped)", pc, condition);
        return;
    }

    if (word & 0x0e00_0010) == 0x0000_0010 {
        match (word >> 5) & 7 {
            0..=3 => dispatch_var_shift(core, pc, word),
            4 => dispatch_swap_mul(core, pc, word),
            5 => dispatch_halfword(core, pc, word),
            6 => dispatch_signed::<0>(core, pc, word),
            7 => dispatch_signed::<1>(core, pc, word),
            _ => unreachable!(),
        }
        return;
    }

    match (word >> 20) & 0xff {
        0x00 => alu_register::<op::And, false, false>(core, pc, word),
        0x01 => alu_register::<op::And, true, false>(core, pc, word),
        0x02 => alu_register::<op::Eor, false, false>(core, pc, word),
        0x03 => alu_register::<op::Eor, true, false>(core, pc, word),
        0x04 => alu_register::<op::Sub, false, false>(core, pc, word),
        0x05 => alu_register::<op::Sub, true, false>(core, pc, word),
        0x06 => alu_register::<op::Rsb, false, false>(core, pc, word),
        0x07 => alu_register::<op::Rsb, true, false>(core, pc, word),

        0x08 => alu_register::<op::Add, false, false>(core, pc, word),
        0x09 => alu_register::<op::Add, true, false>(core, pc, word),
        0x0a => alu_register::<op::Adc, false, false>(core, pc, word),
        0x0b => alu_register::<op::Adc, true, false>(core, pc, word),
        0x0c => alu_register::<op::Sbc, false, false>(core, pc, word),
        0x0d => alu_register::<op::Sbc, true, false>(core, pc, word),
        0x0e => alu_register::<op::Rsc, false, false>(core, pc, word),
        0x0f => alu_register::<op::Rsc, true, false>(core, pc, word),

        0x10 => mrs_register::<false>(core, pc, word),
        0x11 => alu_register::<op::Tst, true, false>(core, pc, word),
        0x12 => msr_register::<false>(core, pc, word),
        0x13 => alu_register::<op::Teq, true, false>(core, pc, word),
        0x14 => mrs_register::<true>(core, pc, word),
        0x15 => alu_register::<op::Cmp, true, false>(core, pc, word),
        0x16 => msr_register::<true>(core, pc, word),
        0x17 => alu_register::<op::Cmn, true, false>(core, pc, word),

        0x18 => alu_register::<op::Orr, false, false>(core, pc, word),
        0x19 => alu_register::<op::Orr, true, false>(core, pc, word),
        0x1a => alu_register::<op::Mov, false, false>(core, pc, word),
        0x1b => alu_register::<op::Mov, true, false>(core, pc, word),
        0x1c => alu_register::<op::Bic, false, false>(core, pc, word),
        0x1d => alu_register::<op::Bic, true, false>(core, pc, word),
        0x1e => alu_register::<op::Mvn, false, false>(core, pc, word),
        0x1f => alu_register::<op::Mvn, true, false>(core, pc, word),

        0x20 => alu_immediate::<op::And, false>(core, pc, word),
        0x21 => alu_immediate::<op::And, true>(core, pc, word),
        0x22 => alu_immediate::<op::Eor, false>(core, pc, word),
        0x23 => alu_immediate::<op::Eor, true>(core, pc, word),
        0x24 => alu_immediate::<op::Sub, false>(core, pc, word),
        0x25 => alu_immediate::<op::Sub, true>(core, pc, word),
        0x26 => alu_immediate::<op::Rsb, false>(core, pc, word),
        0x27 => alu_immediate::<op::Rsb, true>(core, pc, word),

        0x28 => alu_immediate::<op::Add, false>(core, pc, word),
        0x29 => alu_immediate::<op::Add, true>(core, pc, word),
        0x2a => alu_immediate::<op::Adc, false>(core, pc, word),
        0x2b => alu_immediate::<op::Adc, true>(core, pc, word),
        0x2c => alu_immediate::<op::Sbc, false>(core, pc, word),
        0x2d => alu_immediate::<op::Sbc, true>(core, pc, word),
        0x2e => alu_immediate::<op::Rsc, false>(core, pc, word),
        0x2f => alu_immediate::<op::Rsc, true>(core, pc, word),

        0x31 => alu_immediate::<op::Tst, true>(core, pc, word),
        0x33 => alu_immediate::<op::Teq, true>(core, pc, word),
        0x35 => alu_immediate::<op::Cmp, true>(core, pc, word),
        0x37 => alu_immediate::<op::Cmn, true>(core, pc, word),

        0x38 => alu_immediate::<op::Orr, false>(core, pc, word),
        0x39 => alu_immediate::<op::Orr, true>(core, pc, word),
        0x3a => alu_immediate::<op::Mov, false>(core, pc, word),
        0x3b => alu_immediate::<op::Mov, true>(core, pc, word),
        0x3c => alu_immediate::<op::Bic, false>(core, pc, word),
        0x3d => alu_immediate::<op::Bic, true>(core, pc, word),
        0x3e => alu_immediate::<op::Mvn, false>(core, pc, word),
        0x3f => alu_immediate::<op::Mvn, true>(core, pc, word),

        0x40 => mem_immediate::<op::Str, 2, 0b000>(core, pc, word),
        0x41 => mem_immediate::<op::Ldr, 2, 0b000>(core, pc, word),
        0x42 => mem_immediate::<op::Str, 2, 0b001>(core, pc, word),
        0x43 => mem_immediate::<op::Ldr, 2, 0b001>(core, pc, word),
        0x44 => mem_immediate::<op::Str, 0, 0b000>(core, pc, word),
        0x45 => mem_immediate::<op::Ldr, 0, 0b000>(core, pc, word),
        0x46 => mem_immediate::<op::Str, 0, 0b001>(core, pc, word),
        0x47 => mem_immediate::<op::Ldr, 0, 0b001>(core, pc, word),

        0x48 => mem_immediate::<op::Str, 2, 0b010>(core, pc, word),
        0x49 => mem_immediate::<op::Ldr, 2, 0b010>(core, pc, word),
        0x4a => mem_immediate::<op::Str, 2, 0b011>(core, pc, word),
        0x4b => mem_immediate::<op::Ldr, 2, 0b011>(core, pc, word),
        0x4c => mem_immediate::<op::Str, 0, 0b010>(core, pc, word),
        0x4d => mem_immediate::<op::Ldr, 0, 0b010>(core, pc, word),
        0x4e => mem_immediate::<op::Str, 0, 0b011>(core, pc, word),
        0x4f => mem_immediate::<op::Ldr, 0, 0b011>(core, pc, word),

        0x50 => mem_immediate::<op::Str, 2, 0b100>(core, pc, word),
        0x51 => mem_immediate::<op::Ldr, 2, 0b100>(core, pc, word),
        0x52 => mem_immediate::<op::Str, 2, 0b101>(core, pc, word),
        0x53 => mem_immediate::<op::Ldr, 2, 0b101>(core, pc, word),
        0x54 => mem_immediate::<op::Str, 0, 0b100>(core, pc, word),
        0x55 => mem_immediate::<op::Ldr, 0, 0b100>(core, pc, word),
        0x56 => mem_immediate::<op::Str, 0, 0b101>(core, pc, word),
        0x57 => mem_immediate::<op::Ldr, 0, 0b101>(core, pc, word),

        0x58 => mem_immediate::<op::Str, 2, 0b110>(core, pc, word),
        0x59 => mem_immediate::<op::Ldr, 2, 0b110>(core, pc, word),
        0x5a => mem_immediate::<op::Str, 2, 0b111>(core, pc, word),
        0x5b => mem_immediate::<op::Ldr, 2, 0b111>(core, pc, word),
        0x5c => mem_immediate::<op::Str, 0, 0b110>(core, pc, word),
        0x5d => mem_immediate::<op::Ldr, 0, 0b110>(core, pc, word),
        0x5e => mem_immediate::<op::Str, 0, 0b111>(core, pc, word),
        0x5f => mem_immediate::<op::Ldr, 0, 0b111>(core, pc, word),

        0x60 => mem_register::<op::Str, 2, 0b000>(core, pc, word),
        0x61 => mem_register::<op::Ldr, 2, 0b000>(core, pc, word),
        0x62 => mem_register::<op::Str, 2, 0b001>(core, pc, word),
        0x63 => mem_register::<op::Ldr, 2, 0b001>(core, pc, word),
        0x64 => mem_register::<op::Str, 0, 0b000>(core, pc, word),
        0x65 => mem_register::<op::Ldr, 0, 0b000>(core, pc, word),
        0x66 => mem_register::<op::Str, 0, 0b001>(core, pc, word),
        0x67 => mem_register::<op::Ldr, 0, 0b001>(core, pc, word),

        0x68 => mem_register::<op::Str, 2, 0b010>(core, pc, word),
        0x69 => mem_register::<op::Ldr, 2, 0b010>(core, pc, word),
        0x6a => mem_register::<op::Str, 2, 0b011>(core, pc, word),
        0x6b => mem_register::<op::Ldr, 2, 0b011>(core, pc, word),
        0x6c => mem_register::<op::Str, 0, 0b010>(core, pc, word),
        0x6d => mem_register::<op::Ldr, 0, 0b010>(core, pc, word),
        0x6e => mem_register::<op::Str, 0, 0b011>(core, pc, word),
        0x6f => mem_register::<op::Ldr, 0, 0b011>(core, pc, word),

        0x70 => mem_register::<op::Str, 2, 0b100>(core, pc, word),
        0x71 => mem_register::<op::Ldr, 2, 0b100>(core, pc, word),
        0x72 => mem_register::<op::Str, 2, 0b101>(core, pc, word),
        0x73 => mem_register::<op::Ldr, 2, 0b101>(core, pc, word),
        0x74 => mem_register::<op::Str, 0, 0b100>(core, pc, word),
        0x75 => mem_register::<op::Ldr, 0, 0b100>(core, pc, word),
        0x76 => mem_register::<op::Str, 0, 0b101>(core, pc, word),
        0x77 => mem_register::<op::Ldr, 0, 0b101>(core, pc, word),

        0x78 => mem_register::<op::Str, 2, 0b110>(core, pc, word),
        0x79 => mem_register::<op::Ldr, 2, 0b110>(core, pc, word),
        0x7a => mem_register::<op::Str, 2, 0b111>(core, pc, word),
        0x7b => mem_register::<op::Ldr, 2, 0b111>(core, pc, word),
        0x7c => mem_register::<op::Str, 0, 0b110>(core, pc, word),
        0x7d => mem_register::<op::Ldr, 0, 0b110>(core, pc, word),
        0x7e => mem_register::<op::Str, 0, 0b111>(core, pc, word),
        0x7f => mem_register::<op::Ldr, 0, 0b111>(core, pc, word),

        0x80 => stm::<0b00, false, false>(core, pc, word),
        0x81 => ldm::<0b00, false, false>(core, pc, word),
        0x82 => stm::<0b00, false, true>(core, pc, word),
        0x83 => ldm::<0b00, false, true>(core, pc, word),
        0x84 => stm::<0b00, true, false>(core, pc, word),
        0x85 => ldm::<0b00, true, false>(core, pc, word),
        0x86 => stm::<0b00, true, true>(core, pc, word),
        0x87 => ldm::<0b00, true, true>(core, pc, word),

        0x88 => stm::<0b01, false, false>(core, pc, word),
        0x89 => ldm::<0b01, false, false>(core, pc, word),
        0x8a => stm::<0b01, false, true>(core, pc, word),
        0x8b => ldm::<0b01, false, true>(core, pc, word),
        0x8c => stm::<0b01, true, false>(core, pc, word),
        0x8d => ldm::<0b01, true, false>(core, pc, word),
        0x8e => stm::<0b01, true, true>(core, pc, word),
        0x8f => ldm::<0b01, true, true>(core, pc, word),

        0x90 => stm::<0b10, false, false>(core, pc, word),
        0x91 => ldm::<0b10, false, false>(core, pc, word),
        0x92 => stm::<0b10, false, true>(core, pc, word),
        0x93 => ldm::<0b10, false, true>(core, pc, word),
        0x94 => stm::<0b10, true, false>(core, pc, word),
        0x95 => ldm::<0b10, true, false>(core, pc, word),
        0x96 => stm::<0b10, true, true>(core, pc, word),
        0x97 => ldm::<0b10, true, true>(core, pc, word),

        0x98 => stm::<0b11, false, false>(core, pc, word),
        0x99 => ldm::<0b11, false, false>(core, pc, word),
        0x9a => stm::<0b11, false, true>(core, pc, word),
        0x9b => ldm::<0b11, false, true>(core, pc, word),
        0x9c => stm::<0b11, true, false>(core, pc, word),
        0x9d => ldm::<0b11, true, false>(core, pc, word),
        0x9e => stm::<0b11, true, true>(core, pc, word),
        0x9f => ldm::<0b11, true, true>(core, pc, word),

        0xa0..=0xaf => branch::<false>(core, pc, word),
        0xb0..=0xbf => branch::<true>(core, pc, word),

        opcode => todo!("ARM7 Opcode {0:02X} [{0:08b}] (PC: {1:08X})", opcode, pc),
    }
}

fn dispatch_var_shift(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    match (word >> 20) & 0x1f {
        0x00 => alu_register::<op::And, false, true>(core, pc, word),
        0x01 => alu_register::<op::And, true, true>(core, pc, word),
        0x02 => alu_register::<op::Eor, false, true>(core, pc, word),
        0x03 => alu_register::<op::Eor, true, true>(core, pc, word),
        0x04 => alu_register::<op::Sub, false, true>(core, pc, word),
        0x05 => alu_register::<op::Sub, true, true>(core, pc, word),
        0x06 => alu_register::<op::Rsb, false, true>(core, pc, word),
        0x07 => alu_register::<op::Rsb, true, true>(core, pc, word),

        0x08 => alu_register::<op::Add, false, true>(core, pc, word),
        0x09 => alu_register::<op::Add, true, true>(core, pc, word),
        0x0a => alu_register::<op::Adc, false, true>(core, pc, word),
        0x0b => alu_register::<op::Adc, true, true>(core, pc, word),
        0x0c => alu_register::<op::Sbc, false, true>(core, pc, word),
        0x0d => alu_register::<op::Sbc, true, true>(core, pc, word),
        0x0e => alu_register::<op::Rsc, false, true>(core, pc, word),
        0x0f => alu_register::<op::Rsc, true, true>(core, pc, word),

        0x11 => alu_register::<op::Tst, true, true>(core, pc, word),
        0x12 => bx(core, pc, word),
        0x13 => alu_register::<op::Teq, true, true>(core, pc, word),
        0x15 => alu_register::<op::Cmp, true, true>(core, pc, word),
        0x17 => alu_register::<op::Cmn, true, true>(core, pc, word),
        0x18 => alu_register::<op::Orr, false, true>(core, pc, word),

        0x19 => alu_register::<op::Orr, true, true>(core, pc, word),
        0x1a => alu_register::<op::Mov, false, true>(core, pc, word),
        0x1b => alu_register::<op::Mov, true, true>(core, pc, word),
        0x1c => alu_register::<op::Bic, false, true>(core, pc, word),
        0x1d => alu_register::<op::Bic, true, true>(core, pc, word),
        0x1e => alu_register::<op::Mvn, false, true>(core, pc, word),
        0x1f => alu_register::<op::Mvn, true, true>(core, pc, word),

        opcode => todo!(
            "ARM7 Var Shift Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
            opcode,
            pc
        ),
    }
}

fn dispatch_swap_mul(_core: &mut Core<impl Bus>, pc: u32, word: u32) {
    match (word >> 20) & 0x1f {
        opcode => todo!(
            "ARM7 Swap/Mul Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
            opcode,
            pc
        ),
    }
}

fn dispatch_halfword(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    match (word >> 20) & 0x1f {
        0x00 => mem_register::<op::Str, 1, 0b000>(core, pc, word),
        0x01 => mem_register::<op::Ldr, 1, 0b000>(core, pc, word),
        0x02 => mem_register::<op::Str, 1, 0b001>(core, pc, word),
        0x03 => mem_register::<op::Ldr, 1, 0b001>(core, pc, word),
        0x04 => mem_immediate::<op::Str, 1, 0b000>(core, pc, word),
        0x05 => mem_immediate::<op::Ldr, 1, 0b000>(core, pc, word),
        0x06 => mem_immediate::<op::Str, 1, 0b001>(core, pc, word),
        0x07 => mem_immediate::<op::Ldr, 1, 0b001>(core, pc, word),

        0x08 => mem_register::<op::Str, 1, 0b010>(core, pc, word),
        0x09 => mem_register::<op::Ldr, 1, 0b010>(core, pc, word),
        0x0a => mem_register::<op::Str, 1, 0b011>(core, pc, word),
        0x0b => mem_register::<op::Ldr, 1, 0b011>(core, pc, word),
        0x0c => mem_immediate::<op::Str, 1, 0b010>(core, pc, word),
        0x0d => mem_immediate::<op::Ldr, 1, 0b010>(core, pc, word),
        0x0e => mem_immediate::<op::Str, 1, 0b011>(core, pc, word),
        0x0f => mem_immediate::<op::Ldr, 1, 0b011>(core, pc, word),

        0x10 => mem_register::<op::Str, 1, 0b100>(core, pc, word),
        0x11 => mem_register::<op::Ldr, 1, 0b100>(core, pc, word),
        0x12 => mem_register::<op::Str, 1, 0b101>(core, pc, word),
        0x13 => mem_register::<op::Ldr, 1, 0b101>(core, pc, word),
        0x14 => mem_immediate::<op::Str, 1, 0b100>(core, pc, word),
        0x15 => mem_immediate::<op::Ldr, 1, 0b100>(core, pc, word),
        0x16 => mem_immediate::<op::Str, 1, 0b101>(core, pc, word),
        0x17 => mem_immediate::<op::Ldr, 1, 0b101>(core, pc, word),

        0x18 => mem_register::<op::Str, 1, 0b110>(core, pc, word),
        0x19 => mem_register::<op::Ldr, 1, 0b110>(core, pc, word),
        0x1a => mem_register::<op::Str, 1, 0b111>(core, pc, word),
        0x1b => mem_register::<op::Ldr, 1, 0b111>(core, pc, word),
        0x1c => mem_immediate::<op::Str, 1, 0b110>(core, pc, word),
        0x1d => mem_immediate::<op::Ldr, 1, 0b110>(core, pc, word),
        0x1e => mem_immediate::<op::Str, 1, 0b111>(core, pc, word),
        0x1f => mem_immediate::<op::Ldr, 1, 0b111>(core, pc, word),

        opcode => todo!(
            "ARM7 Halfword Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
            opcode,
            pc
        ),
    }
}

fn dispatch_signed<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    match (word >> 20) & 0x1f {
        0x01 => mem_register::<op::Lds, SIZE, 0b000>(core, pc, word),
        0x03 => mem_register::<op::Lds, SIZE, 0b001>(core, pc, word),
        0x05 => mem_immediate::<op::Lds, SIZE, 0b000>(core, pc, word),
        0x07 => mem_immediate::<op::Lds, SIZE, 0b001>(core, pc, word),

        0x09 => mem_register::<op::Lds, SIZE, 0b010>(core, pc, word),
        0x0b => mem_register::<op::Lds, SIZE, 0b011>(core, pc, word),
        0x0d => mem_immediate::<op::Lds, SIZE, 0b010>(core, pc, word),
        0x0f => mem_immediate::<op::Lds, SIZE, 0b011>(core, pc, word),

        0x11 => mem_register::<op::Lds, SIZE, 0b100>(core, pc, word),
        0x13 => mem_register::<op::Lds, SIZE, 0b101>(core, pc, word),
        0x15 => mem_immediate::<op::Lds, SIZE, 0b100>(core, pc, word),
        0x17 => mem_immediate::<op::Lds, SIZE, 0b101>(core, pc, word),

        0x19 => mem_register::<op::Lds, SIZE, 0b110>(core, pc, word),
        0x1b => mem_register::<op::Lds, SIZE, 0b111>(core, pc, word),
        0x1d => mem_immediate::<op::Lds, SIZE, 0b110>(core, pc, word),
        0x1f => mem_immediate::<op::Lds, SIZE, 0b111>(core, pc, word),

        opcode => todo!(
            "ARM7 Signed Halfword Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
            opcode,
            pc
        ),
    }
}

fn apply_shift<const SET_FLAGS: bool, const VAR_SHIFT: bool, const SET_CARRY: bool>(
    core: &mut Core<impl Bus>,
    rm: usize,
    shift_type: usize,
    shift_amount: u32,
) -> u32 {
    match shift_type {
        0b00 => op::Lsl::apply::<SET_FLAGS, VAR_SHIFT, SET_CARRY>(core, core.get(rm), shift_amount),
        0b01 => op::Lsr::apply::<SET_FLAGS, VAR_SHIFT, SET_CARRY>(core, core.get(rm), shift_amount),
        0b10 => op::Asr::apply::<SET_FLAGS, VAR_SHIFT, SET_CARRY>(core, core.get(rm), shift_amount),
        0b11 => op::Ror::apply::<SET_FLAGS, VAR_SHIFT, SET_CARRY>(core, core.get(rm), shift_amount),
        _ => unreachable!(),
    }
}

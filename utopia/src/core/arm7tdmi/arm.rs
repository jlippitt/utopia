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
            0 | 1 | 2 | 3 => dispatch_var_shift(core, pc, word),
            4 => dispatch_swap_mul(core, pc, word),
            5 => dispatch_halfword(core, pc, word),
            6 => dispatch_signed::<0>(core, pc, word),
            7 => dispatch_signed::<1>(core, pc, word),
            _ => unreachable!(),
        }
        return;
    }

    match (word >> 20) & 0xff {
        0x00 => binary_register::<op::And, false, false>(core, pc, word),
        0x01 => binary_register::<op::And, true, false>(core, pc, word),
        0x02 => binary_register::<op::Eor, false, false>(core, pc, word),
        0x03 => binary_register::<op::Eor, true, false>(core, pc, word),
        0x04 => binary_register::<op::Sub, false, false>(core, pc, word),
        0x05 => binary_register::<op::Sub, true, false>(core, pc, word),
        0x06 => binary_register::<op::Rsb, false, false>(core, pc, word),
        0x07 => binary_register::<op::Rsb, true, false>(core, pc, word),

        0x08 => binary_register::<op::Add, false, false>(core, pc, word),
        0x09 => binary_register::<op::Add, true, false>(core, pc, word),
        0x0a => binary_register::<op::Adc, false, false>(core, pc, word),
        0x0b => binary_register::<op::Adc, true, false>(core, pc, word),
        0x0c => binary_register::<op::Sbc, false, false>(core, pc, word),
        0x0d => binary_register::<op::Sbc, true, false>(core, pc, word),
        0x0e => binary_register::<op::Rsc, false, false>(core, pc, word),
        0x0f => binary_register::<op::Rsc, true, false>(core, pc, word),

        0x11 => compare_register::<op::Tst, false>(core, pc, word),
        0x12 => msr_register::<false>(core, pc, word),
        0x13 => compare_register::<op::Teq, false>(core, pc, word),
        0x15 => compare_register::<op::Cmp, false>(core, pc, word),
        0x16 => msr_register::<true>(core, pc, word),
        0x17 => compare_register::<op::Cmn, false>(core, pc, word),

        0x18 => binary_register::<op::Orr, false, false>(core, pc, word),
        0x19 => binary_register::<op::Orr, true, false>(core, pc, word),
        0x1a => move_register::<op::Mov, false, false>(core, pc, word),
        0x1b => move_register::<op::Mov, true, false>(core, pc, word),
        0x1c => binary_register::<op::Bic, false, false>(core, pc, word),
        0x1d => binary_register::<op::Bic, true, false>(core, pc, word),
        0x1e => move_register::<op::Mvn, false, false>(core, pc, word),
        0x1f => move_register::<op::Mvn, true, false>(core, pc, word),

        0x20 => binary_immediate::<op::And, false>(core, pc, word),
        0x21 => binary_immediate::<op::And, true>(core, pc, word),
        0x22 => binary_immediate::<op::Eor, false>(core, pc, word),
        0x23 => binary_immediate::<op::Eor, true>(core, pc, word),
        0x24 => binary_immediate::<op::Sub, false>(core, pc, word),
        0x25 => binary_immediate::<op::Sub, true>(core, pc, word),
        0x26 => binary_immediate::<op::Rsb, false>(core, pc, word),
        0x27 => binary_immediate::<op::Rsb, true>(core, pc, word),

        0x28 => binary_immediate::<op::Add, false>(core, pc, word),
        0x29 => binary_immediate::<op::Add, true>(core, pc, word),
        0x2a => binary_immediate::<op::Adc, false>(core, pc, word),
        0x2b => binary_immediate::<op::Adc, true>(core, pc, word),
        0x2c => binary_immediate::<op::Sbc, false>(core, pc, word),
        0x2d => binary_immediate::<op::Sbc, true>(core, pc, word),
        0x2e => binary_immediate::<op::Rsc, false>(core, pc, word),
        0x2f => binary_immediate::<op::Rsc, true>(core, pc, word),

        0x31 => compare_immediate::<op::Tst>(core, pc, word),
        0x33 => compare_immediate::<op::Teq>(core, pc, word),
        0x35 => compare_immediate::<op::Cmp>(core, pc, word),
        0x37 => compare_immediate::<op::Cmn>(core, pc, word),

        0x38 => binary_immediate::<op::Orr, false>(core, pc, word),
        0x39 => binary_immediate::<op::Orr, true>(core, pc, word),
        0x3a => move_immediate::<op::Mov, false>(core, pc, word),
        0x3b => move_immediate::<op::Mov, true>(core, pc, word),
        0x3c => binary_immediate::<op::Bic, false>(core, pc, word),
        0x3d => binary_immediate::<op::Bic, true>(core, pc, word),
        0x3e => move_immediate::<op::Mvn, false>(core, pc, word),
        0x3f => move_immediate::<op::Mvn, true>(core, pc, word),

        0x40 => str_immediate::<2, 0b000>(core, pc, word),
        0x41 => ldr_immediate::<2, 0b000>(core, pc, word),
        0x42 => str_immediate::<2, 0b001>(core, pc, word),
        0x43 => ldr_immediate::<2, 0b001>(core, pc, word),
        0x44 => str_immediate::<0, 0b000>(core, pc, word),
        0x45 => ldr_immediate::<0, 0b000>(core, pc, word),
        0x46 => str_immediate::<0, 0b001>(core, pc, word),
        0x47 => ldr_immediate::<0, 0b001>(core, pc, word),

        0x48 => str_immediate::<2, 0b010>(core, pc, word),
        0x49 => ldr_immediate::<2, 0b010>(core, pc, word),
        0x4a => str_immediate::<2, 0b011>(core, pc, word),
        0x4b => ldr_immediate::<2, 0b011>(core, pc, word),
        0x4c => str_immediate::<0, 0b010>(core, pc, word),
        0x4d => ldr_immediate::<0, 0b010>(core, pc, word),
        0x4e => str_immediate::<0, 0b011>(core, pc, word),
        0x4f => ldr_immediate::<0, 0b011>(core, pc, word),

        0x50 => str_immediate::<2, 0b100>(core, pc, word),
        0x51 => ldr_immediate::<2, 0b100>(core, pc, word),
        0x52 => str_immediate::<2, 0b101>(core, pc, word),
        0x53 => ldr_immediate::<2, 0b101>(core, pc, word),
        0x54 => str_immediate::<0, 0b100>(core, pc, word),
        0x55 => ldr_immediate::<0, 0b100>(core, pc, word),
        0x56 => str_immediate::<0, 0b101>(core, pc, word),
        0x57 => ldr_immediate::<0, 0b101>(core, pc, word),

        0x58 => str_immediate::<2, 0b110>(core, pc, word),
        0x59 => ldr_immediate::<2, 0b110>(core, pc, word),
        0x5a => str_immediate::<2, 0b111>(core, pc, word),
        0x5b => ldr_immediate::<2, 0b111>(core, pc, word),
        0x5c => str_immediate::<0, 0b110>(core, pc, word),
        0x5d => ldr_immediate::<0, 0b110>(core, pc, word),
        0x5e => str_immediate::<0, 0b111>(core, pc, word),
        0x5f => ldr_immediate::<0, 0b111>(core, pc, word),

        0x60 => str_register::<2, 0b000>(core, pc, word),
        0x61 => ldr_register::<2, 0b000>(core, pc, word),
        0x62 => str_register::<2, 0b001>(core, pc, word),
        0x63 => ldr_register::<2, 0b001>(core, pc, word),
        0x64 => str_register::<0, 0b000>(core, pc, word),
        0x65 => ldr_register::<0, 0b000>(core, pc, word),
        0x66 => str_register::<0, 0b001>(core, pc, word),
        0x67 => ldr_register::<0, 0b001>(core, pc, word),

        0x68 => str_register::<2, 0b010>(core, pc, word),
        0x69 => ldr_register::<2, 0b010>(core, pc, word),
        0x6a => str_register::<2, 0b011>(core, pc, word),
        0x6b => ldr_register::<2, 0b011>(core, pc, word),
        0x6c => str_register::<0, 0b010>(core, pc, word),
        0x6d => ldr_register::<0, 0b010>(core, pc, word),
        0x6e => str_register::<0, 0b011>(core, pc, word),
        0x6f => ldr_register::<0, 0b011>(core, pc, word),

        0x70 => str_register::<2, 0b100>(core, pc, word),
        0x71 => ldr_register::<2, 0b100>(core, pc, word),
        0x72 => str_register::<2, 0b101>(core, pc, word),
        0x73 => ldr_register::<2, 0b101>(core, pc, word),
        0x74 => str_register::<0, 0b100>(core, pc, word),
        0x75 => ldr_register::<0, 0b100>(core, pc, word),
        0x76 => str_register::<0, 0b101>(core, pc, word),
        0x77 => ldr_register::<0, 0b101>(core, pc, word),

        0x78 => str_register::<2, 0b110>(core, pc, word),
        0x79 => ldr_register::<2, 0b110>(core, pc, word),
        0x7a => str_register::<2, 0b111>(core, pc, word),
        0x7b => ldr_register::<2, 0b111>(core, pc, word),
        0x7c => str_register::<0, 0b110>(core, pc, word),
        0x7d => ldr_register::<0, 0b110>(core, pc, word),
        0x7e => str_register::<0, 0b111>(core, pc, word),
        0x7f => ldr_register::<0, 0b111>(core, pc, word),

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
        0x00 => binary_register::<op::And, false, true>(core, pc, word),
        0x01 => binary_register::<op::And, true, true>(core, pc, word),
        0x02 => binary_register::<op::Eor, false, true>(core, pc, word),
        0x03 => binary_register::<op::Eor, true, true>(core, pc, word),
        0x04 => binary_register::<op::Sub, false, true>(core, pc, word),
        0x05 => binary_register::<op::Sub, true, true>(core, pc, word),
        0x06 => binary_register::<op::Rsb, false, true>(core, pc, word),
        0x07 => binary_register::<op::Rsb, true, true>(core, pc, word),

        0x08 => binary_register::<op::Add, false, true>(core, pc, word),
        0x09 => binary_register::<op::Add, true, true>(core, pc, word),
        0x0a => binary_register::<op::Adc, false, true>(core, pc, word),
        0x0b => binary_register::<op::Adc, true, true>(core, pc, word),
        0x0c => binary_register::<op::Sbc, false, true>(core, pc, word),
        0x0d => binary_register::<op::Sbc, true, true>(core, pc, word),
        0x0e => binary_register::<op::Rsc, false, true>(core, pc, word),
        0x0f => binary_register::<op::Rsc, true, true>(core, pc, word),

        0x11 => compare_register::<op::Tst, true>(core, pc, word),
        0x12 => bx(core, pc, word),
        0x13 => compare_register::<op::Teq, true>(core, pc, word),
        0x15 => compare_register::<op::Cmp, true>(core, pc, word),
        0x17 => compare_register::<op::Cmn, true>(core, pc, word),
        0x18 => binary_register::<op::Orr, false, true>(core, pc, word),

        0x19 => binary_register::<op::Orr, true, true>(core, pc, word),
        0x1a => move_register::<op::Mov, false, true>(core, pc, word),
        0x1b => move_register::<op::Mov, true, true>(core, pc, word),
        0x1c => binary_register::<op::Bic, false, true>(core, pc, word),
        0x1d => binary_register::<op::Bic, true, true>(core, pc, word),
        0x1e => move_register::<op::Mvn, false, true>(core, pc, word),
        0x1f => move_register::<op::Mvn, true, true>(core, pc, word),

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
        0x00 => str_register::<1, 0b000>(core, pc, word),
        0x01 => ldr_register::<1, 0b000>(core, pc, word),
        0x02 => str_register::<1, 0b001>(core, pc, word),
        0x03 => ldr_register::<1, 0b001>(core, pc, word),
        0x04 => str_immediate::<1, 0b000>(core, pc, word),
        0x05 => ldr_immediate::<1, 0b000>(core, pc, word),
        0x06 => str_immediate::<1, 0b001>(core, pc, word),
        0x07 => ldr_immediate::<1, 0b001>(core, pc, word),

        0x08 => str_register::<1, 0b010>(core, pc, word),
        0x09 => ldr_register::<1, 0b010>(core, pc, word),
        0x0a => str_register::<1, 0b011>(core, pc, word),
        0x0b => ldr_register::<1, 0b011>(core, pc, word),
        0x0c => str_immediate::<1, 0b010>(core, pc, word),
        0x0d => ldr_immediate::<1, 0b010>(core, pc, word),
        0x0e => str_immediate::<1, 0b011>(core, pc, word),
        0x0f => ldr_immediate::<1, 0b011>(core, pc, word),

        0x10 => str_register::<1, 0b100>(core, pc, word),
        0x11 => ldr_register::<1, 0b100>(core, pc, word),
        0x12 => str_register::<1, 0b101>(core, pc, word),
        0x13 => ldr_register::<1, 0b101>(core, pc, word),
        0x14 => str_immediate::<1, 0b100>(core, pc, word),
        0x15 => ldr_immediate::<1, 0b100>(core, pc, word),
        0x16 => str_immediate::<1, 0b101>(core, pc, word),
        0x17 => ldr_immediate::<1, 0b101>(core, pc, word),

        0x18 => str_register::<1, 0b110>(core, pc, word),
        0x19 => ldr_register::<1, 0b110>(core, pc, word),
        0x1a => str_register::<1, 0b111>(core, pc, word),
        0x1b => ldr_register::<1, 0b111>(core, pc, word),
        0x1c => str_immediate::<1, 0b110>(core, pc, word),
        0x1d => ldr_immediate::<1, 0b110>(core, pc, word),
        0x1e => str_immediate::<1, 0b111>(core, pc, word),
        0x1f => ldr_immediate::<1, 0b111>(core, pc, word),

        opcode => todo!(
            "ARM7 Halfword Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
            opcode,
            pc
        ),
    }
}

fn dispatch_signed<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    match (word >> 20) & 0x1f {
        0x01 => lds_register::<SIZE, 0b000>(core, pc, word),
        0x03 => lds_register::<SIZE, 0b001>(core, pc, word),
        0x05 => lds_immediate::<SIZE, 0b000>(core, pc, word),
        0x07 => lds_immediate::<SIZE, 0b001>(core, pc, word),

        0x09 => lds_register::<SIZE, 0b010>(core, pc, word),
        0x0b => lds_register::<SIZE, 0b011>(core, pc, word),
        0x0d => lds_immediate::<SIZE, 0b010>(core, pc, word),
        0x0f => lds_immediate::<SIZE, 0b011>(core, pc, word),

        0x11 => lds_register::<SIZE, 0b100>(core, pc, word),
        0x13 => lds_register::<SIZE, 0b101>(core, pc, word),
        0x15 => lds_immediate::<SIZE, 0b100>(core, pc, word),
        0x17 => lds_immediate::<SIZE, 0b101>(core, pc, word),

        0x19 => lds_register::<SIZE, 0b110>(core, pc, word),
        0x1b => lds_register::<SIZE, 0b111>(core, pc, word),
        0x1d => lds_immediate::<SIZE, 0b110>(core, pc, word),
        0x1f => lds_immediate::<SIZE, 0b111>(core, pc, word),

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

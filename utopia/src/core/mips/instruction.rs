use super::operator;
use super::{cop0, cop1, Bus, Core};
use control::*;
use immediate::*;
use load::*;
use misc::*;
use mul_div::*;
use register::*;
use store::*;

mod control;
mod immediate;
mod load;
mod misc;
mod mul_div;
mod register;
mod store;

pub fn dispatch(core: &mut Core<impl Bus>, word: u32) {
    use operator as op;

    match word >> 26 {
        0o00 => special(core, word),
        0o01 => regimm(core, word),
        0o03 => type_j(core, jal, word),
        0o04 => type_i(core, branch::<op::Beq, false, false>, word),
        0o05 => type_i(core, branch::<op::Bne, false, false>, word),
        0o06 => type_i(core, branch::<op::Blez, false, false>, word),
        0o07 => type_i(core, branch::<op::Bgtz, false, false>, word),
        0o10 => type_i(core, addi, word),
        0o11 => type_i(core, addiu, word),
        0o12 => type_i(core, slti, word),
        0o13 => type_i(core, sltiu, word),
        0o14 => type_i(core, andi, word),
        0o15 => type_i(core, ori, word),
        0o16 => type_i(core, xori, word),
        0o17 => type_i(core, lui, word),
        0o20 => cop0::dispatch(core, word),
        0o21 => cop1::dispatch(core, word),
        0o24 => type_i(core, branch::<op::Beq, false, true>, word),
        0o25 => type_i(core, branch::<op::Bne, false, true>, word),
        0o26 => type_i(core, branch::<op::Blez, false, true>, word),
        0o27 => type_i(core, branch::<op::Bgtz, false, true>, word),
        0o43 => type_i(core, lw, word),
        0o44 => type_i(core, lbu, word),
        0o50 => type_i(core, sb, word),
        0o53 => type_i(core, sw, word),
        0o57 => type_i(core, cache, word),
        opcode => unimplemented!("Opcode {:02o} ({:08X}: {:08X})", opcode, core.pc, word),
    }
}

fn special(core: &mut Core<impl Bus>, word: u32) {
    match word & 0o77 {
        0o00 => type_r(core, sll, word),
        0o02 => type_r(core, srl, word),
        0o04 => type_r(core, sllv, word),
        0o06 => type_r(core, srlv, word),
        0o10 => type_r(core, jr, word),
        0o20 => type_r(core, mfhi, word),
        0o22 => type_r(core, mflo, word),
        0o31 => type_r(core, multu, word),
        0o40 => type_r(core, add, word),
        0o41 => type_r(core, addu, word),
        0o42 => type_r(core, sub, word),
        0o43 => type_r(core, subu, word),
        0o44 => type_r(core, and, word),
        0o45 => type_r(core, or, word),
        0o46 => type_r(core, xor, word),
        0o52 => type_r(core, slt, word),
        0o53 => type_r(core, sltu, word),
        function => unimplemented!(
            "SPECIAL FN={:02o} ({:08X}: {:08X})",
            function,
            core.pc,
            word,
        ),
    }
}

fn regimm(core: &mut Core<impl Bus>, word: u32) {
    use operator as op;

    match (word >> 16) & 31 {
        0b00000 => type_i(core, branch::<op::Bltz, false, false>, word),
        0b00001 => type_i(core, branch::<op::Bgez, false, false>, word),
        0b00010 => type_i(core, branch::<op::Bltz, false, true>, word),
        0b00011 => type_i(core, branch::<op::Bgez, false, true>, word),
        0b10000 => type_i(core, branch::<op::Bltz, true, false>, word),
        0b10001 => type_i(core, branch::<op::Bgez, true, false>, word),
        0b10010 => type_i(core, branch::<op::Bltz, true, true>, word),
        0b10011 => type_i(core, branch::<op::Bgez, true, true>, word),
        rt => unimplemented!("REGIMM RT={:05b} ({:08X}: {:08X})", rt, core.pc, word),
    }
}

fn type_r<T: Bus>(
    core: &mut Core<T>,
    instr: impl Fn(&mut Core<T>, usize, usize, usize, u32),
    word: u32,
) {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    let sa = (word >> 6) & 31;
    instr(core, rs, rt, rd, sa);
}

fn type_i<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize, u32), word: u32) {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let value = word & 0xffff;
    instr(core, rs, rt, value);
}

fn type_j<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, u32), word: u32) {
    let value = word & 0x03ff_ffff;
    instr(core, value);
}

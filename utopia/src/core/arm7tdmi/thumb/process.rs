use super::super::operator::{self, AluOperator, ShiftOperator};
use super::super::{Bus, Core, REGS};
use tracing::trace;

pub fn move_shifted<Op: ShiftOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let shift_amount = ((word >> 6) & 31) as u32;
    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    trace!(
        "{:08X} {} {}, {}, #0x{:X}",
        pc,
        Op::NAME,
        REGS[rd],
        REGS[rs],
        shift_amount
    );

    let result = Op::apply::<true, false, true>(core, core.get(rs), shift_amount);
    core.set(rd, result);
}

pub fn alu_register_3op<Op: AluOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rn = ((word >> 6) & 7) as usize;
    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    trace!(
        "{:08X} {} {}, {}, {}",
        pc,
        Op::NAME,
        REGS[rd],
        REGS[rs],
        REGS[rn]
    );

    Op::apply::<true>(core, rd, core.get(rs), core.get(rn));
}

pub fn alu_immediate_3op<Op: AluOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let value = (word >> 6) & 7;
    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    trace!(
        "{:08X} {} {}, {}, #{}",
        pc,
        Op::NAME,
        REGS[rd],
        REGS[rs],
        value
    );

    Op::apply::<true>(core, rd, core.get(rs), value as u32);
}

pub fn alu_immediate_2op<Op: AluOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    trace!("{:08X} {} {}, #0x{:02X}", pc, Op::NAME, REGS[rd], value);

    Op::apply::<true>(core, rd, core.get(rd), value as u32);
}

pub fn add_sp_immediate(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let sign = (word & 0x80) != 0;
    let offset = (word & 0x7f) << 2;

    trace!(
        "{:08X} {} {}, #0x{:X}",
        pc,
        if sign { "SUB" } else { "ADD " },
        REGS[13],
        offset
    );

    if sign {
        core.set(13, core.get(13).wrapping_sub(offset as u32));
    } else {
        core.set(13, core.get(13).wrapping_add(offset as u32));
    }
}

pub fn alu_register_high<Op: AluOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rs = ((word >> 3) & 15) as usize;
    let rd = (((word >> 4) & 8) | (word & 7)) as usize;

    trace!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    Op::apply::<false>(core, rd, core.get(rd), core.get(rs));
}

pub fn alu_register_2op(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    use operator as op;

    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    match (word >> 6) & 15 {
        0b0000 => alu_op::<op::And>(core, pc, rs, rd),
        0b0001 => alu_op::<op::Eor>(core, pc, rs, rd),
        0b0010 => shift_op::<op::Lsl>(core, pc, rs, rd),
        0b0011 => shift_op::<op::Lsr>(core, pc, rs, rd),
        0b0100 => shift_op::<op::Asr>(core, pc, rs, rd),
        0b0101 => alu_op::<op::Adc>(core, pc, rs, rd),
        0b0110 => alu_op::<op::Sbc>(core, pc, rs, rd),
        0b0111 => shift_op::<op::Ror>(core, pc, rs, rd),
        0b1000 => alu_op::<op::Tst>(core, pc, rs, rd),
        // 0b1001 => alu_op::<op::Neg>(core, pc, rs, rd),
        0b1010 => alu_op::<op::Cmp>(core, pc, rs, rd),
        0b1011 => alu_op::<op::Cmn>(core, pc, rs, rd),
        0b1100 => alu_op::<op::Orr>(core, pc, rs, rd),
        0b1101 => alu_op::<op::Mul>(core, pc, rs, rd),
        0b1110 => alu_op::<op::Bic>(core, pc, rs, rd),
        0b1111 => alu_op::<op::Mvn>(core, pc, rs, rd),
        opcode => todo!("ALU operation {:04b}", opcode),
        //_ => unreachable!(),
    }
}

fn alu_op<Op: AluOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    trace!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    Op::apply::<true>(core, rd, core.get(rd), core.get(rs));
}

fn shift_op<Op: ShiftOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    trace!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    let result = Op::apply::<true, true, true>(core, core.get(rd), core.get(rs));
    core.set(rd, result);
}

use super::super::operator::{
    self, Add, BinaryOperator, Cmp, CompareOperator, Mov, MoveOperator, MultiplyOperator,
    ShiftOperator,
};
use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn move_shifted<Op: ShiftOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let shift_amount = ((word >> 6) & 31) as u32;
    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} {} {}, {}, #0x{:X}",
        pc,
        Op::NAME,
        REGS[rd],
        REGS[rs],
        shift_amount
    );

    let result = Op::apply::<true, true>(core, core.get(rs), shift_amount);
    core.set(rd, result);
}

pub fn binary_register_3op<Op: BinaryOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rn = ((word >> 6) & 7) as usize;
    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} {} {}, {}, {}",
        pc,
        Op::NAME,
        REGS[rd],
        REGS[rs],
        REGS[rn]
    );

    let result = Op::apply::<true>(core, core.get(rs), core.get(rn));
    core.set(rd, result);
}

pub fn binary_immediate_3op<Op: BinaryOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let value = (word >> 6) & 7;
    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} {} {}, {}, #{}",
        pc,
        Op::NAME,
        REGS[rd],
        REGS[rs],
        value
    );

    let result = Op::apply::<true>(core, core.get(rs), value as u32);
    core.set(rd, result);
}

pub fn binary_immediate<Op: BinaryOperator>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    debug!("{:08X} {} {}, #0x{:02X}", pc, Op::NAME, REGS[rd], value);

    let result = Op::apply::<true>(core, core.get(rd), value as u32);
    core.set(rd, result);
}

pub fn move_immediate(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    debug!("{:08X} MOV {}, #0x{:02X}", pc, REGS[rd], value);

    let result = Mov::apply::<true>(core, value as u32);
    core.set(rd, result);
}

pub fn compare_immediate(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let value = word & 0xff;

    debug!("{:08X} CMP {}, #0x{:02X}", pc, REGS[rd], value);

    Cmp::apply(core, core.get(rd), value as u32);
}

pub fn add_sp_immediate(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let sign = (word & 0x80) != 0;
    let offset = (word & 0x7f) << 2;

    debug!(
        "{:08X} ADD {}, #{}0x{:X}",
        pc,
        REGS[13],
        if sign { "-" } else { "" },
        offset
    );

    if sign {
        core.set(13, core.get(13).wrapping_sub(offset as u32));
    } else {
        core.set(13, core.get(13).wrapping_add(offset as u32));
    }
}

pub fn add_high(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rs = ((word >> 3) & 15) as usize;
    let rd = (((word >> 4) & 8) | (word & 7)) as usize;

    debug!("{:08X} ADD {}, {}", pc, REGS[rd], REGS[rs]);
    let result = Add::apply::<false>(core, core.get(rd), core.get(rs));
    core.set(rd, result);
}

pub fn mov_high(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rs = ((word >> 3) & 15) as usize;
    let rd = (((word >> 4) & 8) | (word & 7)) as usize;

    debug!("{:08X} MOV {}, {}", pc, REGS[rd], REGS[rs]);
    let result = Mov::apply::<false>(core, core.get(rs));
    core.set(rd, result);
}

pub fn cmp_high(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rs = ((word >> 3) & 15) as usize;
    let rd = (((word >> 4) & 8) | (word & 7)) as usize;

    debug!("{:08X} CMP {}, {}", pc, REGS[rd], REGS[rs]);
    Cmp::apply(core, core.get(rd), core.get(rs));
}

pub fn alu_operation(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    use operator as op;

    let rs = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    match (word >> 6) & 15 {
        0b0000 => binary_op::<op::And>(core, pc, rs, rd),
        0b0001 => binary_op::<op::Eor>(core, pc, rs, rd),
        0b0010 => shift_op::<op::Lsl>(core, pc, rs, rd),
        0b0011 => shift_op::<op::Lsr>(core, pc, rs, rd),
        // 0b0100 => shift_op::<op::Asr>(core, pc, rs, rd),
        0b0101 => binary_op::<op::Adc>(core, pc, rs, rd),
        0b0110 => binary_op::<op::Sbc>(core, pc, rs, rd),
        0b0111 => shift_op::<op::Ror>(core, pc, rs, rd),
        0b1000 => compare_op::<op::Tst>(core, pc, rs, rd),
        // 0b1001 => move_op::<op::Neg>(core, pc, rs, rd),
        0b1010 => compare_op::<op::Cmp>(core, pc, rs, rd),
        0b1011 => compare_op::<op::Cmn>(core, pc, rs, rd),
        0b1100 => binary_op::<op::Orr>(core, pc, rs, rd),
        0b1101 => multiply_op::<op::Mul>(core, pc, rs, rd),
        0b1110 => binary_op::<op::Bic>(core, pc, rs, rd),
        0b1111 => move_op::<op::Mvn>(core, pc, rs, rd),
        opcode => todo!("ALU operation {:04b}", opcode),
        //_ => unreachable!(),
    }
}

fn move_op<Op: MoveOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    debug!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    let result = Op::apply::<true>(core, core.get(rs));
    core.set(rd, result);
}

fn compare_op<Op: CompareOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    debug!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    Op::apply(core, core.get(rd), core.get(rs));
}

fn binary_op<Op: BinaryOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    debug!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    let result = Op::apply::<true>(core, core.get(rd), core.get(rs));
    core.set(rd, result);
}

fn multiply_op<Op: MultiplyOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    debug!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    let result = Op::apply::<true>(core, core.get(rd), core.get(rs), 0);
    core.set(rd, result);
}

fn shift_op<Op: ShiftOperator>(core: &mut Core<impl Bus>, pc: u32, rs: usize, rd: usize) {
    debug!("{:08X} {} {}, {}", pc, Op::NAME, REGS[rd], REGS[rs]);
    let result = Op::apply::<true, true>(core, core.get(rd), core.get(rs));
    core.set(rd, result);
}

use super::super::operator::{AluOperator, OpType};
use super::super::{Bus, Core, REGS};
use super::{apply_shift, SHIFT};
use tracing::debug;

pub fn alu_immediate<Op: AluOperator, const SET_FLAGS: bool>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let value = (word & 0xff).rotate_right(((word >> 8) & 15) << 1);

    match Op::TYPE {
        OpType::Binary => debug!(
            "{:08X} {}{} {}, {}, #0x{:08X}",
            pc,
            Op::NAME,
            if SET_FLAGS { "S" } else { "" },
            REGS[rd],
            REGS[rn],
            value
        ),
        OpType::Move => debug!(
            "{:08X} {}{} {}, #0x{:08X}",
            pc,
            Op::NAME,
            if SET_FLAGS { "S" } else { "" },
            REGS[rd],
            value
        ),
        OpType::Compare => debug!("{:08X} {} {}, #0x{:08X}", pc, Op::NAME, REGS[rn], value),
    }

    if SET_FLAGS && rd == 15 {
        core.cpsr_from_u32(core.spsr_to_u32(), true);
        Op::apply::<false>(core, rd, core.get(rn), value);
    }

    Op::apply::<SET_FLAGS>(core, rd, core.get(rn), value);
}

pub fn alu_register<Op: AluOperator, const SET_FLAGS: bool, const VAR_SHIFT: bool>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let rm = (word & 15) as usize;
    let shift_type = ((word >> 5) & 3) as usize;

    let (shift_amount, debug_string) = if VAR_SHIFT {
        let rs = ((word >> 8) & 15) as usize;
        (core.get(rs), REGS[rs].to_string())
    } else {
        let shift_amount = (word >> 7) & 31;
        (shift_amount, format!("#0x{:X}", shift_amount))
    };

    match Op::TYPE {
        OpType::Binary => debug!(
            "{:08X} {}{} {}, {}, {}, {} {}",
            pc,
            Op::NAME,
            if SET_FLAGS { "S" } else { "" },
            REGS[rd],
            REGS[rn],
            REGS[rm],
            SHIFT[shift_type],
            debug_string
        ),
        OpType::Move => debug!(
            "{:08X} {}{} {}, {}, {} {}",
            pc,
            Op::NAME,
            if SET_FLAGS { "S" } else { "" },
            REGS[rd],
            REGS[rm],
            SHIFT[shift_type],
            debug_string
        ),
        OpType::Compare => debug!(
            "{:08X} {} {}, {}, {} {}",
            pc,
            Op::NAME,
            REGS[rn],
            REGS[rm],
            SHIFT[shift_type],
            debug_string
        ),
    }

    let value = if Op::LOGICAL {
        apply_shift::<SET_FLAGS, VAR_SHIFT, true>(core, rm, shift_type, shift_amount)
    } else {
        apply_shift::<SET_FLAGS, VAR_SHIFT, false>(core, rm, shift_type, shift_amount)
    };

    if SET_FLAGS && rd == 15 {
        core.cpsr_from_u32(core.spsr_to_u32(), true);
        Op::apply::<false>(core, rd, core.get(rn), value);
    };

    Op::apply::<SET_FLAGS>(core, rd, core.get(rn), value);
}

pub fn mrs_register<const SPSR: bool>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rd = ((word >> 12) & 15) as usize;

    debug!(
        "{:08X} MRS {}, {}PSR",
        pc,
        REGS[rd],
        if SPSR { "S" } else { "C" },
    );

    let result = if SPSR {
        core.spsr_to_u32()
    } else {
        core.cpsr_to_u32()
    };

    core.set(rd, result);
}

pub fn msr_register<const SPSR: bool>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rm = (word & 15) as usize;
    let control = (word & 0x0001_0000) != 0;

    debug!(
        "{:08X} MSR {}PSR_F{}, {}",
        pc,
        if SPSR { "S" } else { "C" },
        if control { "C" } else { "" },
        REGS[rm],
    );

    if SPSR {
        core.spsr_from_u32(core.get(rm), control);
    } else {
        core.cpsr_from_u32(core.get(rm), control);
    }
}

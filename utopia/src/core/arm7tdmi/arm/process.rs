use super::super::operator::{BinaryOperator, ComparisonOperator, MoveOperator};
use super::super::{Bus, Core, REGS};
use tracing::debug;

fn immediate_value(word: u32) -> u32 {
    (word & 0xff).rotate_right(((word >> 8) & 15) << 1)
}

pub fn binary_immediate<Op: BinaryOperator, const SET_FLAGS: bool>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let value = immediate_value(word);

    debug!(
        "{:08X} {}{} {}, {}, #0x{:08X}",
        pc,
        Op::NAME,
        if SET_FLAGS { "S" } else { "" },
        REGS[rd],
        REGS[rn],
        value
    );

    let result = if SET_FLAGS && rd == 15 {
        todo!("Weird PC register flag handling");
    } else {
        Op::apply::<SET_FLAGS>(core, core.get(rn), value)
    };

    core.set(rd, result);
}

pub fn move_immediate<Op: MoveOperator, const SET_FLAGS: bool>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rd = ((word >> 12) & 15) as usize;
    let value = immediate_value(word);

    debug!(
        "{:08X} {}{} {}, #0x{:08X}",
        pc,
        Op::NAME,
        if SET_FLAGS { "S" } else { "" },
        REGS[rd],
        value
    );

    let result = if SET_FLAGS && rd == 15 {
        todo!("Weird PC register flag handling");
    } else {
        Op::apply::<SET_FLAGS>(core, value)
    };

    core.set(rd, result);
}

pub fn compare_immediate<Op: ComparisonOperator>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rn = ((word >> 16) & 15) as usize;
    let value = immediate_value(word);

    debug!("{:08X} {} {}, #0x{:08X}", pc, Op::NAME, REGS[rn], value);

    Op::apply(core, core.get(rn), value);
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

use super::super::operator::{ComparisonOperator, MoveOperator};
use super::super::{Bus, Core, REGS};
use tracing::debug;

fn immediate_value(word: u32) -> u32 {
    (word & 0xff).rotate_right(((word >> 8) & 15) << 1)
}

pub fn move_immediate<Op: MoveOperator, const SET_FLAGS: bool>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rd = ((word >> 12) & 15) as usize;
    let value = immediate_value(word);

    debug!(
        "{:08X} {}{} {}, #{:08X}",
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

    debug!("{:08X} {} {}, #{:08X}", pc, Op::NAME, REGS[rn], value);

    Op::apply(core, core.get(rn), value);
}

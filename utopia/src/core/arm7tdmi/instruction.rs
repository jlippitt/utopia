use super::operator::ComparisonOperator;
use super::{Bus, Core};
use tracing::debug;

fn immediate_value(word: u32) -> u32 {
    (word & 0xff).rotate_right(((word >> 8) & 15) << 1)
}

pub fn branch<const LINK: bool>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let offset = ((word as i32) << 8) >> 6;

    debug!("{:08X} B{} {:+}", pc, if LINK { "L" } else { "" }, offset);

    if LINK {
        core.regs[14] = core.pc;
    }

    core.pc = core.pc.wrapping_add(4).wrapping_add(offset as u32);
}

pub fn compare_immediate<Op: ComparisonOperator>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rn = (word >> 16) & 15;
    let value = immediate_value(word);
    debug!("{:08X} {} R{}, #{}", pc, Op::NAME, rn, value);
    Op::apply(core, core.get(rn), value);
}

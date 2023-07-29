use super::super::operator::BranchOperator;
use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn branch<Op: BranchOperator, const LIKELY: bool>(
    core: &mut Core<impl Bus>,
    rs: usize,
    rt: usize,
    value: u32,
) {
    let offset = (value as i16 as i32) << 2;

    debug!(
        "{:08X} {}{} {}, {}, {}",
        core.pc,
        Op::NAME,
        if LIKELY { "L" } else { "" },
        REGS[rt],
        REGS[rs],
        offset
    );

    if Op::apply(core.get(rs), core.get(rt)) {
        debug!("  Branch taken");
        core.next[1] = core.next[0].wrapping_add(offset as u32);
    } else {
        debug!("  Branch not taken");

        if LIKELY {
            // Skip the delay slot
            core.next[0] = core.next[1];
            core.next[1] = core.next[1].wrapping_add(4);
        }
    }
}

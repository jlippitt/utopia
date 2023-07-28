use super::super::{Bus, Core};
use super::{regs, NAME};
use tracing::debug;

pub fn mtc<const COP: usize>(core: &mut Core<impl Bus>, word: u32) {
    let (_, rt, rd) = regs(word);
    debug!("{:08X} MTC{} {}, ${}", core.pc, COP, NAME[rt], rd);
    // Nothing for now
}

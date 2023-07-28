use super::super::{Bus, Core};
use super::NAME;
use tracing::debug;

pub fn mtc<const COP: usize>(core: &mut Core<impl Bus>, _rs: usize, rt: usize, rd: usize) {
    debug!("{:08X} MTC{} {}, ${}", core.pc, COP, NAME[rt], rd);
    // Nothing for now
}

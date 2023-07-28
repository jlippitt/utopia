use super::super::{Bus, Core};
use tracing::debug;

pub fn jal(core: &mut Core<impl Bus>, value: u32) {
    let target = (core.next[0] & 0xfc00_0000) | (value << 2);
    debug!("{:08X} JAL 0x{:08X}", core.pc, target);
    core.set(31, core.next[1]);
    core.next[1] = target;
}

use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn jal(core: &mut Core<impl Bus>, value: u32) {
    let target = (core.next[0] & 0xfc00_0000) | (value << 2);
    debug!("{:08X} JAL 0x{:08X}", core.pc, target);
    core.set(31, core.next[1]);
    core.next[1] = target;
}

pub fn jr(core: &mut Core<impl Bus>, rs: usize, _rt: usize, _rd: usize, _sa: u32) {
    debug!("{:08X} JR {}", core.pc, REGS[rs]);
    core.next[1] = core.get(rs);
}

use super::super::{Bus, Core};
use super::NAME;
use tracing::debug;

pub fn lui(core: &mut Core<impl Bus>, _rs: usize, rt: usize, value: u32) {
    debug!("{:08X} LUI {}, 0x{:04X}", core.pc, NAME[rt], value);
    core.regs[rt] = value << 16;
    debug!("  {}: {:08X}", NAME[rt], core.regs[rt]);
}

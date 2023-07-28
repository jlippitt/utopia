use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn lui(core: &mut Core<impl Bus>, _rs: usize, rt: usize, value: u32) {
    debug!("{:08X} LUI {}, 0x{:04X}", core.pc, REGS[rt], value);
    core.regs[rt] = value << 16;
    debug!("  {}: {:08X}", REGS[rt], core.regs[rt]);
}

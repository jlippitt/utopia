use super::super::{Bus, Core};
use tracing::debug;

pub fn c_le_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, _fd: usize) {
    debug!("{:08X} C.LE.S $F{}, $F{}", core.pc, fs, ft);
    core.cp1.set_c(core.cp1.s(fs) < core.cp1.s(ft));
}

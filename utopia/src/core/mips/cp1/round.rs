use super::super::{Bus, Core};
use tracing::debug;

pub fn trunc_w_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    debug!("{:08X} TRUNC.W.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_w(fd, core.cp1.s(fs).trunc() as i32);
}

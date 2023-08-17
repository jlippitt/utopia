use super::super::{Bus, Core};
use tracing::debug;

pub fn cvt_d_w(core: &mut Core<impl Bus>, fs: usize, fd: usize) {
    debug!("{:08X} CVT.D.W $F{}, $F{}", core.pc, fd, fs);
    core.cp1.setd(fd, core.cp1.getw(fs) as f64);
}

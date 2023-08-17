use super::super::{Bus, Core};
use tracing::debug;

// TODO: Rounding modes

pub fn cvt_s_w(core: &mut Core<impl Bus>, fs: usize, fd: usize) {
    debug!("{:08X} CVT.S.W $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, core.cp1.w(fs) as f32);
}

pub fn cvt_d_w(core: &mut Core<impl Bus>, fs: usize, fd: usize) {
    debug!("{:08X} CVT.D.W $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_d(fd, core.cp1.w(fs) as f64);
}

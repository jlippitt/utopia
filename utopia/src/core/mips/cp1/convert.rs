use super::super::{Bus, Core};
use tracing::trace;

// TODO: Rounding modes for f64 -> f32 conversions(?)

pub fn cvt_s_d(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} CVT.S.D $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, core.cp1.d(fs) as f32);
}

pub fn cvt_s_w(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} CVT.S.W $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, core.cp1.w(fs) as f32);
}

pub fn cvt_d_w(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} CVT.D.W $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_d(fd, core.cp1.w(fs) as f64);
}

pub fn cvt_w_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} CVT.W.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_w(fd, core.cp1.round(core.cp1.s(fs)) as i32);
}

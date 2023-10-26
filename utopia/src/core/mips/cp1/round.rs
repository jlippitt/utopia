use super::super::{Bus, Core};
use tracing::trace;

pub fn round_w_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} ROUND.W.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_w(fd, core.cp1.s(fs).round() as i32);
}

pub fn trunc_w_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} TRUNC.W.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_w(fd, core.cp1.s(fs).trunc() as i32);
}

pub fn ceil_w_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} CEIL.W.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_w(fd, core.cp1.s(fs).ceil() as i32);
}

pub fn floor_w_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} FLOOR.W.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_w(fd, core.cp1.s(fs).floor() as i32);
}

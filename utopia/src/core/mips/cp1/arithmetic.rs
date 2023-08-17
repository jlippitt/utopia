use super::super::{Bus, Core};
use tracing::debug;

// TODO: Rounding modes

pub fn add_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, fd: usize) {
    debug!("{:08X} ADD.S $F{}, $F{}, $F{}", core.pc, fd, fs, ft);
    core.cp1.set_s(fd, core.cp1.s(fs) + core.cp1.s(ft));
}

pub fn div_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, fd: usize) {
    debug!("{:08X} DIV.S $F{}, $F{}, $F{}", core.pc, fd, fs, ft);
    core.cp1.set_s(fd, core.cp1.s(fs) / core.cp1.s(ft));
}

use super::super::{Bus, Core};
use tracing::trace;

// TODO: Rounding modes

pub fn add_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} ADD.S $F{}, $F{}, $F{}", core.pc, fd, fs, ft);
    core.cp1.set_s(fd, core.cp1.s(fs) + core.cp1.s(ft));
}

pub fn sub_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} SUB.S $F{}, $F{}, $F{}", core.pc, fd, fs, ft);
    core.cp1.set_s(fd, core.cp1.s(fs) - core.cp1.s(ft));
}

pub fn mul_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} MUL.S $F{}, $F{}, $F{}", core.pc, fd, fs, ft);
    core.cp1.set_s(fd, core.cp1.s(fs) * core.cp1.s(ft));
}

pub fn div_s(core: &mut Core<impl Bus>, ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} DIV.S $F{}, $F{}, $F{}", core.pc, fd, fs, ft);
    core.cp1.set_s(fd, core.cp1.s(fs) / core.cp1.s(ft));
}

pub fn sqrt_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} SQRT.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, core.cp1.s(fs).sqrt());
}

pub fn abs_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} ABS.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, core.cp1.s(fs).abs());
}

pub fn mov_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} MOV.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, core.cp1.s(fs));
}

pub fn neg_s(core: &mut Core<impl Bus>, _ft: usize, fs: usize, fd: usize) {
    trace!("{:08X} NEG.S $F{}, $F{}", core.pc, fd, fs);
    core.cp1.set_s(fd, -core.cp1.s(fs));
}

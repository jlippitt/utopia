use super::{Bus, Core, Cp1, Opcode};
use tracing::trace;

// Note: MIPS handles .5 rounding differently to Rust's 'round()' method. To fix
// this, we need https://github.com/rust-lang/rust/issues/96710 to be stabilised.

pub fn round_w_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} ROUND.W.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.gets(op.fs()).round() as i32);
}

pub fn trunc_w_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} TRUNC.W.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.gets(op.fs()).trunc() as i32);
}

pub fn ceil_w_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CEIL.W.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.gets(op.fs()).ceil() as i32);
}

pub fn floor_w_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} FLOOR.W.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.gets(op.fs()).floor() as i32);
}

pub fn round_w_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} ROUND.W.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.getd(op.fs()).round() as i32);
}

pub fn trunc_w_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} TRUNC.W.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.getd(op.fs()).trunc() as i32);
}

pub fn ceil_w_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CEIL.W.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.getd(op.fs()).ceil() as i32);
}

pub fn floor_w_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} FLOOR.W.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.getd(op.fs()).floor() as i32);
}

pub fn round_l_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} ROUND.L.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.gets(op.fs()).round() as i64);
}

pub fn trunc_l_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} TRUNC.L.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.gets(op.fs()).trunc() as i64);
}

pub fn ceil_l_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CEIL.L.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.gets(op.fs()).ceil() as i64);
}

pub fn floor_l_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} FLOOR.L.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.gets(op.fs()).floor() as i64);
}

pub fn round_l_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} ROUND.L.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.getd(op.fs()).round() as i64);
}

pub fn trunc_l_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} TRUNC.L.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.getd(op.fs()).trunc() as i64);
}

pub fn ceil_l_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CEIL.L.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.getd(op.fs()).ceil() as i64);
}

pub fn floor_l_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} FLOOR.L.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.getd(op.fs()).floor() as i64);
}

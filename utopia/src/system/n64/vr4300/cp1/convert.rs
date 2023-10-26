use super::{Bus, Core, Cp1, Opcode};
use tracing::trace;

pub fn cvt_s_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.S.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.getd(op.fs()) as f32);
}

pub fn cvt_s_w(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.S.W F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.getw(op.fs()) as f32);
}

pub fn cvt_s_l(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.S.L F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.getl(op.fs()) as f32);
}

pub fn cvt_d_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.D.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.gets(op.fs()) as f64);
}

pub fn cvt_d_w(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.D.W F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getw(op.fs()) as f64);
}

pub fn cvt_d_l(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.D.L F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getl(op.fs()) as f64);
}

pub fn cvt_w_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.W.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.gets(op.fs()) as i32);
}

pub fn cvt_w_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.W.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setw(op.fd(), cp1.getd(op.fs()) as i32);
}

pub fn cvt_l_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.L.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.gets(op.fs()) as i64);
}

pub fn cvt_l_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} CVT.L.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setl(op.fd(), cp1.getd(op.fs()) as i64);
}

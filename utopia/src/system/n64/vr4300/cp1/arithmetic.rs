use super::{Bus, Core, Cp1, Opcode};
use tracing::trace;

pub fn add_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} ADD.S F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()) + cp1.gets(op.ft()));
}

pub fn sub_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} SUB.S F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()) - cp1.gets(op.ft()));
}

pub fn mul_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} MUL.S F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()) * cp1.gets(op.ft()));
}

pub fn div_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} DIV.S F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()) / cp1.gets(op.ft()));
}

pub fn sqrt_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} SQRT.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()).sqrt());
}

pub fn abs_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} ABS.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()).abs());
}

pub fn mov_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} MOV.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), cp1.gets(op.fs()));
}

pub fn neg_s(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} NEG.S F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.sets(op.fd(), -cp1.gets(op.fs()));
}

pub fn add_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} ADD.D F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()) + cp1.getd(op.ft()));
}

pub fn sub_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} SUB.D F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()) - cp1.getd(op.ft()));
}

pub fn mul_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} MUL.D F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()) * cp1.getd(op.ft()));
}

pub fn div_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);

    trace!(
        "{:08X} DIV.D F{}, F{}, F{}",
        core.pc(),
        op.fd(),
        op.fs(),
        op.ft()
    );

    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()) / cp1.getd(op.ft()));
}

pub fn sqrt_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} SQRT.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()).sqrt());
}

pub fn abs_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} ABS.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()).abs());
}

pub fn mov_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} MOV.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), cp1.getd(op.fs()));
}

pub fn neg_d(core: &mut Core<impl Bus<Cp1 = Cp1>>, word: u32) {
    let op = Opcode::from(word);
    trace!("{:08X} NEG.D F{}, F{}", core.pc(), op.fd(), op.fs());
    let cp1 = core.cp1_mut();
    cp1.setd(op.fd(), -cp1.getd(op.fs()));
}

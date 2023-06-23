use super::super::{Core, Bus, ReadAddress, WriteAddress};
use tracing::debug;

fn bit_from_opcode(opcode: u8) -> u8 {
   (opcode >> 3) & 7
}

pub fn rla(core: &mut Core<impl Bus>) {
    debug!("RLA");
    let carry = core.flags.c as u8;
    core.flags.c = (core.a & 0x80) != 0;
    core.a = (core.a << 1) | carry;
    core.flags.z = 0xff;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rl<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("RL {}", Addr::NAME);
    let value = Addr::read(core);
    let carry = core.flags.c as u8;
    core.flags.c = (value & 0x80) != 0;
    let result = (value << 1) | carry;
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn bit<Addr: ReadAddress<u8>>(core: &mut Core<impl Bus>, opcode: u8) {
    let bit = bit_from_opcode(opcode);
    debug!("BIT {}, {}", bit, Addr::NAME);
    core.flags.z = Addr::read(core) & (1 << bit);
    core.flags.n = false;
    core.flags.h = true;
}
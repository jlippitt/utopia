use super::super::{Bus, Core, ReadAddress, WriteAddress};
use tracing::trace;

fn bit_from_opcode(opcode: u8) -> u8 {
    (opcode >> 3) & 7
}

pub fn rlca(core: &mut Core<impl Bus>) {
    trace!("RLCA");
    core.flags.c = (core.a & 0x80) != 0;
    core.a = (core.a << 1) | (core.a >> 7);
    core.flags.z = 0xff;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rrca(core: &mut Core<impl Bus>) {
    trace!("RRCA");
    core.flags.c = (core.a & 0x01) != 0;
    core.a = (core.a >> 1) | (core.a << 7);
    core.flags.z = 0xff;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rla(core: &mut Core<impl Bus>) {
    trace!("RLA");
    let carry = core.flags.c as u8;
    core.flags.c = (core.a & 0x80) != 0;
    core.a = (core.a << 1) | carry;
    core.flags.z = 0xff;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rra(core: &mut Core<impl Bus>) {
    trace!("RRA");
    let carry = core.flags.c as u8;
    core.flags.c = (core.a & 0x01) != 0;
    core.a = (core.a >> 1) | (carry << 7);
    core.flags.z = 0xff;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rlc<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("RLC {}", Addr::NAME);
    let value = Addr::read(core);
    core.flags.c = (value & 0x80) != 0;
    let result = (value << 1) | (value >> 7);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rrc<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("RRC {}", Addr::NAME);
    let value = Addr::read(core);
    core.flags.c = (value & 0x01) != 0;
    let result = (value >> 1) | (value << 7);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rl<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("RL {}", Addr::NAME);
    let value = Addr::read(core);
    let carry = core.flags.c as u8;
    core.flags.c = (value & 0x80) != 0;
    let result = (value << 1) | carry;
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn rr<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("RR {}", Addr::NAME);
    let value = Addr::read(core);
    let carry = core.flags.c as u8;
    core.flags.c = (value & 0x01) != 0;
    let result = (value >> 1) | (carry << 7);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn sla<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("SLA {}", Addr::NAME);
    let value = Addr::read(core);
    core.flags.c = (value & 0x80) != 0;
    let result = value << 1;
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn sra<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("SRA {}", Addr::NAME);
    let value = Addr::read(core);
    core.flags.c = (value & 0x01) != 0;
    let result = (value & 0x80) | (value >> 1);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn swap<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("SWAP {}", Addr::NAME);
    let value = Addr::read(core);
    let result = (value << 4) | (value >> 4);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}

pub fn srl<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("SRL {}", Addr::NAME);
    let value = Addr::read(core);
    core.flags.c = (value & 0x01) != 0;
    let result = value >> 1;
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = false;
}

pub fn bit<Addr: ReadAddress<u8>>(core: &mut Core<impl Bus>, opcode: u8) {
    let bit = bit_from_opcode(opcode);
    trace!("BIT {}, {}", bit, Addr::NAME);
    core.flags.z = Addr::read(core) & (1 << bit);
    core.flags.n = false;
    core.flags.h = true;
}

pub fn res<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>, opcode: u8) {
    let bit = bit_from_opcode(opcode);
    trace!("RES {}, {}", bit, Addr::NAME);
    let result = Addr::read(core) & !(1 << bit);
    Addr::write(core, result);
}

pub fn set<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>, opcode: u8) {
    let bit = bit_from_opcode(opcode);
    trace!("SET {}, {}", bit, Addr::NAME);
    let result = Addr::read(core) | (1 << bit);
    Addr::write(core, result);
}

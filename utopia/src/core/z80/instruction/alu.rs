use super::super::address_mode::{ReadAddress, WriteAddress};
use super::super::{Bus, Core};
use tracing::trace;

fn add_with_carry(core: &mut Core<impl Bus>, value: u8, carry: bool) -> u8 {
    let result = core.a.wrapping_add(value).wrapping_add(carry as u8);
    let carries = core.a ^ value ^ result;
    let overflow = (core.a ^ result) & (value ^ result);
    core.set_sz(result);
    core.flags.n = false;
    core.flags.h = (carries & 0x10) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

fn subtract_with_borrow(core: &mut Core<impl Bus>, value: u8, borrow: bool) -> u8 {
    let result = core.a.wrapping_sub(value).wrapping_sub(borrow as u8);
    let carries = core.a ^ value ^ result;
    let overflow = (core.a ^ result) & (value ^ core.a);
    core.set_sz(result);
    core.flags.n = true;
    core.flags.h = (carries & 0x10) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

pub fn add<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("ADD A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = add_with_carry(core, value, false);
}

pub fn adc<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("ADC A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = add_with_carry(core, value, core.flags.c);
}

pub fn sub<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("SUB A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = subtract_with_borrow(core, value, false);
}

pub fn sbc<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("SBC A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = subtract_with_borrow(core, value, core.flags.c);
}

pub fn and<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("AND A, {}", Rhs::NAME);
    core.a &= Rhs::read(core);
    core.set_sz(core.a);
    core.flags.n = false;
    core.flags.h = true;
    core.flags.c = false;
}

pub fn xor<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("XOR A, {}", Rhs::NAME);
    core.a ^= Rhs::read(core);
    core.set_sz(core.a);
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}

pub fn or<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("OR A, {}", Rhs::NAME);
    core.a |= Rhs::read(core);
    core.set_sz(core.a);
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}

pub fn cp<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("CP A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    subtract_with_borrow(core, value, false);
}

pub fn inc<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("INC {}", Addr::NAME);
    let result = Addr::read(core).wrapping_add(1);
    Addr::write(core, result);
    core.set_sz(result);
    core.flags.n = false;
    core.flags.h = (result & 0x0f) == 0;
}

pub fn dec<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    trace!("DEC {}", Addr::NAME);
    let result = Addr::read(core).wrapping_sub(1);
    Addr::write(core, result);
    core.set_sz(result);
    core.flags.n = true;
    core.flags.h = (result & 0x0f) == 0x0f;
}

pub fn add16<Rhs: ReadAddress<u16>>(core: &mut Core<impl Bus>) {
    trace!("ADD HL, {}", Rhs::NAME);
    core.idle();
    let value = Rhs::read(core);
    let result = core.hl.wrapping_add(value);
    let carries = core.hl ^ value ^ result;
    let overflow = (core.hl ^ result) & (value ^ result);
    core.hl = result;
    core.flags.n = false;
    core.flags.c = ((carries ^ overflow) & 0x8000) != 0;
    core.flags.h = (carries & 0x1000) != 0;
}

pub fn inc16<Addr: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    trace!("INC {}", Addr::NAME);
    core.idle();
    let result = Addr::read(core).wrapping_add(1);
    Addr::write(core, result);
}

pub fn dec16<Addr: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    trace!("DEC {}", Addr::NAME);
    core.idle();
    let result = Addr::read(core).wrapping_sub(1);
    Addr::write(core, result);
}

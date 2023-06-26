use super::super::{Bus, Core, ReadAddress, WriteAddress};
use tracing::debug;

fn add_with_carry(core: &mut Core<impl Bus>, value: u8, carry: bool) -> u8 {
    let result = core.a.wrapping_add(value).wrapping_add(carry as u8);
    let carries = core.a ^ value ^ result;
    let overflow = (core.a ^ result) & (value ^ result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = (carries & 0x10) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

fn subtract_with_borrow(core: &mut Core<impl Bus>, value: u8, borrow: bool) -> u8 {
    let result = core.a.wrapping_sub(value).wrapping_sub(borrow as u8);
    let carries = core.a ^ value ^ result;
    let overflow = (core.a ^ result) & (value ^ core.a);
    core.flags.z = result;
    core.flags.n = true;
    core.flags.h = (carries & 0x10) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

fn add_offset_to_sp(core: &mut Core<impl Bus>) -> u16 {
    let offset = ((core.next_byte() as i8) as i16) as u16;
    let result = core.sp.wrapping_add(offset);
    let carries = core.sp ^ offset ^ result;
    let overflow = (core.sp ^ result) & (offset ^ result);
    core.flags.z = 0xff;
    core.flags.n = false;
    core.flags.h = (carries & 0x10) != 0;
    core.flags.c = ((carries ^ overflow) & 0x80) != 0;
    result
}

pub fn add<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("ADD A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = add_with_carry(core, value, false);
}

pub fn adc<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("ADC A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = add_with_carry(core, value, core.flags.c);
}

pub fn sub<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("SUB A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = subtract_with_borrow(core, value, false);
}

pub fn sbc<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("SBC A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    core.a = subtract_with_borrow(core, value, core.flags.c);
}

pub fn and<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("AND A, {}", Rhs::NAME);
    core.a &= Rhs::read(core);
    core.flags.z = core.a;
    core.flags.n = false;
    core.flags.h = true;
    core.flags.c = false;
}

pub fn xor<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("XOR A, {}", Rhs::NAME);
    core.a ^= Rhs::read(core);
    core.flags.z = core.a;
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}

pub fn or<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("OR A, {}", Rhs::NAME);
    core.a |= Rhs::read(core);
    core.flags.z = core.a;
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}

pub fn cp<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("CP A, {}", Rhs::NAME);
    let value = Rhs::read(core);
    subtract_with_borrow(core, value, false);
}

pub fn inc<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("INC {}", Addr::NAME);
    let result = Addr::read(core).wrapping_add(1);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = (result & 0x0f) == 0;
}

pub fn dec<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("DEC {}", Addr::NAME);
    let result = Addr::read(core).wrapping_sub(1);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = true;
    core.flags.h = (result & 0x0f) == 0x0f;
}

pub fn add16<Rhs: ReadAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("ADD HL, {}", Rhs::NAME);
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
    debug!("INC {}", Addr::NAME);
    core.idle();
    let result = Addr::read(core).wrapping_add(1);
    Addr::write(core, result);
}

pub fn dec16<Addr: WriteAddress<u16>>(core: &mut Core<impl Bus>) {
    debug!("DEC {}", Addr::NAME);
    core.idle();
    let result = Addr::read(core).wrapping_sub(1);
    Addr::write(core, result);
}

pub fn add_sp_i8(core: &mut Core<impl Bus>) {
    debug!("ADD SP, i8");
    core.sp = add_offset_to_sp(core);
    core.idle();
    core.idle();
}

pub fn ld_hl_sp_i8(core: &mut Core<impl Bus>) {
    debug!("LD HL, SP+i8");
    core.hl = add_offset_to_sp(core);
    core.idle();
}

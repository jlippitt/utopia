use super::super::address_mode::{AddressMode, DirectIndirect};
use super::super::{Bus, Core};
use tracing::debug;

fn push_register<const E: bool, const MX: bool>(core: &mut Core<impl Bus>, value: u16) {
    core.idle();

    if !MX {
        core.push::<E>((value >> 8) as u8);
    }

    core.poll();
    core.push::<E>(value as u8);
}

fn pull_register<const E: bool, const MX: bool>(core: &mut Core<impl Bus>, mask: u16) -> u16 {
    core.idle();
    core.idle();

    if MX {
        core.poll();
        let value = core.pull::<E>();
        let result = mask | (value as u16);
        core.set_nz8(value);
        result
    } else {
        let low = core.pull::<E>();
        core.poll();
        let high = core.pull::<E>();
        let result = u16::from_le_bytes([low, high]);
        core.set_nz16(result);
        result
    }
}

pub fn php<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PHP");
    core.idle();
    core.poll();
    core.push::<E>(core.flags_to_u8::<E>(true));
}

pub fn plp<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PLP");
    core.idle();
    core.idle();
    core.poll();
    let value = core.pull::<E>();
    core.flags_from_u8::<E>(value);
}

pub fn pha<const E: bool, const M: bool>(core: &mut Core<impl Bus>) {
    debug!("PHA.{}", super::size(M));
    push_register::<E, M>(core, core.a);
}

pub fn pla<const E: bool, const M: bool>(core: &mut Core<impl Bus>) {
    debug!("PLA.{}", super::size(M));
    core.a = pull_register::<E, M>(core, core.a & 0xff00);
}

pub fn phx<const E: bool, const X: bool>(core: &mut Core<impl Bus>) {
    debug!("PHX.{}", super::size(X));
    push_register::<E, X>(core, core.x);
}

pub fn plx<const E: bool, const X: bool>(core: &mut Core<impl Bus>) {
    debug!("PLX.{}", super::size(X));
    core.x = pull_register::<E, X>(core, 0);
}

pub fn phy<const E: bool, const X: bool>(core: &mut Core<impl Bus>) {
    debug!("PHY.{}", super::size(X));
    push_register::<E, X>(core, core.y);
}

pub fn ply<const E: bool, const X: bool>(core: &mut Core<impl Bus>) {
    debug!("PLY.{}", super::size(X));
    core.y = pull_register::<E, X>(core, 0);
}

pub fn phk<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PHK");
    core.idle();
    core.poll();
    core.push::<E>((core.pc >> 16) as u8);
}

pub fn phb<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PHB");
    core.idle();
    core.poll();
    core.push::<E>((core.dbr >> 16) as u8);
}

pub fn plb<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PLB");
    core.idle();
    core.idle();
    core.poll();
    let value = core.pull::<E>();
    core.dbr = (value as u32) << 16;
    core.set_nz8(value);
}

pub fn phd<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PHD");
    core.idle();
    core.push::<E>((core.d >> 8) as u8);
    core.poll();
    core.push::<E>(core.d as u8);
}

pub fn pld<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PLD");
    core.idle();
    core.idle();
    let low = core.pull::<E>();
    core.poll();
    let high = core.pull::<E>();
    core.d = u16::from_le_bytes([low, high]);
    core.set_nz16(core.d);
}

pub fn pea<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PEA addr");
    let target = core.next_word();
    core.push::<E>((target >> 8) as u8);
    core.poll();
    core.push::<E>(target as u8);
}

pub fn pei<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PEI (dp)");
    let target = DirectIndirect::<false>::resolve(core, true);
    core.push::<E>((target >> 8) as u8);
    core.poll();
    core.push::<E>(target as u8);
}

pub fn per<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("PER label");
    let offset = core.next_word();
    let target = (core.pc as u16).wrapping_add(offset);
    core.push::<E>((target >> 8) as u8);
    core.poll();
    core.push::<E>(target as u8);
}

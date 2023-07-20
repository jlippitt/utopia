use super::super::{Bus, Core};
use tracing::debug;

pub fn jmp(core: &mut Core<impl Bus>) {
    debug!("JMP addr");
    let low = core.next_byte();
    core.poll();
    let high = core.next_byte();
    let target = u16::from_le_bytes([low, high]);
    core.pc = (core.pc & 0xffff0000) | (target as u32);
}

pub fn jmp_long(core: &mut Core<impl Bus>) {
    debug!("JMP long");
    let low = core.next_byte();
    let high = core.next_byte();
    core.poll();
    let bank = core.next_byte();
    core.pc = u32::from_le_bytes([low, high, bank, 0]);
}

pub fn jmp_indirect_long(core: &mut Core<impl Bus>) {
    debug!("JMP [addr]");
    let low_address = core.next_word();
    let low = core.read(low_address as u32);
    let high_address = low_address.wrapping_add(1);
    let high = core.read(high_address as u32);
    core.poll();
    let bank_address = high_address.wrapping_add(1);
    let bank = core.read(bank_address as u32);
    core.pc = u32::from_le_bytes([low, high, bank, 0]);
}

pub fn jsr<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("JSR addr");
    let low = core.next_byte();
    let high = core.read(core.pc);
    let target = u16::from_le_bytes([low, high]);
    core.idle();
    core.push::<E>((core.pc >> 8) as u8);
    core.poll();
    core.push::<E>(core.pc as u8);
    core.pc = (core.pc & 0xffff0000) | (target as u32);
}

pub fn jsl<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("JSL long");
    let low = core.next_byte();
    let high = core.next_byte();
    core.push::<E>((core.pc >> 16) as u8);
    core.idle();
    let bank = core.read(core.pc);
    core.push::<E>((core.pc >> 8) as u8);
    core.poll();
    core.push::<E>(core.pc as u8);
    core.pc = u32::from_le_bytes([low, high, bank, 0]);
}

pub fn rts<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("RTS");
    core.idle();
    core.idle();
    let low = core.pull::<E>();
    let high = core.pull::<E>();
    let target = u16::from_le_bytes([low, high]).wrapping_add(1);
    core.pc = (core.pc & 0xffff0000) | (target as u32);
    core.poll();
    core.idle();
}

pub fn rtl<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("RTL");
    core.idle();
    core.idle();
    let low = core.pull::<E>();
    let high = core.pull::<E>();
    let target = u16::from_le_bytes([low, high]).wrapping_add(1);
    core.poll();
    let bank = core.pull::<E>();
    core.pc = ((bank as u32) << 16) | (target as u32);
}

pub fn rti<const E: bool>(core: &mut Core<impl Bus>) {
    debug!("RTI");
    core.idle();
    core.idle();
    let flags = core.pull::<E>();
    core.flags_from_u8::<E>(flags);
    let low = core.pull::<E>();

    if E {
        core.poll();
        let high = core.pull::<E>();
        let target = u16::from_le_bytes([low, high]);
        core.pc = (core.pc & 0xffff0000) | (target as u32);
    } else {
        let high = core.pull::<E>();
        core.poll();
        let bank = core.pull::<E>();
        core.pc = u32::from_le_bytes([low, high, bank, 0]);
    }
}

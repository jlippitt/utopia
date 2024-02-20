use super::{AddressMode, Bus, Core, Size};
use std::mem;
use tracing::trace;

pub fn lea(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    let dst = (word >> 9) & 7;
    trace!("LEA {}, A{}", src, dst);
    let value = src.address(core);
    core.set_areg(dst as usize, value);
}

pub fn move_<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    let dst = AddressMode::from(((word >> 3) & 56) | ((word >> 9) & 7));
    trace!("MOVE.{} {}, {}", T::NAME, src, dst);
    let value: T = src.read(core);
    core.set_ccr(|flags| {
        flags.set_nz(value);
        flags.v = false;
        flags.c = false;
    });
    dst.write(core, value);
}

pub fn moveq(core: &mut Core<impl Bus>, word: u16) {
    let value = (word & 0xff) as i8 as u32;
    let dst = (word >> 9) & 7;
    trace!("MOVEQ #{}, D{}", value, dst);
    core.set_ccr(|flags| {
        flags.set_nz(value);
        flags.v = false;
        flags.c = false;
    });
    core.set_dreg(dst as usize, value);
}

pub fn movea<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    let dst = (word >> 9) & 7;
    trace!("MOVEA.{} {}, A{}", T::NAME, src, dst);
    let value: T = src.read(core);
    core.set_areg(dst as usize, value);
}

pub fn move_to_usp(core: &mut Core<impl Bus>, word: u16) {
    let src = word & 7;
    trace!("MOVE.L A{}, USP", src);
    let value = core.areg(src as usize);
    core.set_usp(value);
}

pub fn movem_read<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    trace!("MOVEM.{} {}, regs", T::NAME, src);
    let reg_mask: u16 = core.next();

    if src.is_pre_decrement() {
        todo!("Pre-decrement MOVEM transfers");
    } else {
        let mut address = src.address(core);

        for index in 0..=7 {
            if reg_mask & (0x0001 << index) != 0 {
                core.set_dreg::<T>(index, core.read(address));
                address = address.wrapping_add(mem::size_of::<T>() as u32);
            }
        }

        for index in 0..=7 {
            if reg_mask & (0x0100 << index) != 0 {
                core.set_areg::<T>(index, core.read(address));
                address = address.wrapping_add(mem::size_of::<T>() as u32);
            }
        }

        if src.is_post_increment() {
            core.set_areg(src.reg(), address);
        }
    }
}

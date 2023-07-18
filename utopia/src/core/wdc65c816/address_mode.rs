use super::{Bus, Core};

const WRAP16: u32 = 0x0000_ffff;
const WRAP24: u32 = 0x00ff_ffff;

fn index_direct<const E: bool>(core: &Core<impl Bus>, base: u32, index: u32) -> u32 {
    if E && (core.d & 0xff) == 0 {
        (base & 0xff00) | (base.wrapping_add(index) & 0xff)
    } else {
        base.wrapping_add(index) & WRAP16
    }
}

pub trait AddressMode {
    const NAME: &'static str;
    const WRAP: u32;
    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32;
}

pub struct Absolute;

impl AddressMode for Absolute {
    const NAME: &'static str = "addr";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        core.dbr | (core.next_word() as u32)
    }
}

pub struct AbsoluteLong;

impl AddressMode for AbsoluteLong {
    const NAME: &'static str = "long";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        core.next_long()
    }
}

pub struct AbsoluteLongX;

impl AddressMode for AbsoluteLongX {
    const NAME: &'static str = "long,X";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        core.next_long().wrapping_add(core.x as u32) & WRAP24
    }
}

pub struct Direct;

impl AddressMode for Direct {
    const NAME: &'static str = "dp";
    const WRAP: u32 = WRAP16;

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        let base = core.next_byte() as u32;

        if (core.d & 0xff) != 0 {
            core.idle();
        }

        base.wrapping_add(core.d as u32) & WRAP16
    }
}

pub struct DirectX<const E: bool>;

impl<const E: bool> AddressMode for DirectX<E> {
    const NAME: &'static str = "dp,X";
    const WRAP: u32 = WRAP16;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = Direct::resolve(core, write);
        core.idle();
        index_direct::<E>(core, base, core.x as u32)
    }
}

pub struct DirectIndirectLong;

impl AddressMode for DirectIndirectLong {
    const NAME: &'static str = "[dp]";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let low_address = Direct::resolve(core, write);
        let low = core.read(low_address);
        let high_address = low_address.wrapping_add(1) & WRAP16;
        let high = core.read(high_address);
        let bank_address = high_address.wrapping_add(1) & WRAP16;
        let bank = core.read(bank_address);
        u32::from_le_bytes([low, high, bank, 0])
    }
}

pub struct DirectIndirectLongY;

impl AddressMode for DirectIndirectLongY {
    const NAME: &'static str = "[dp],Y";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = DirectIndirectLong::resolve(core, write);
        base.wrapping_add(core.y as u32) & WRAP24
    }
}

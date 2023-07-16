use super::{Bus, Core};

const WRAP24: u32 = 0x00ff_ffff;

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

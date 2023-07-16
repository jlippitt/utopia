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

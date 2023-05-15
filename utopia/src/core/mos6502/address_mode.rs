use super::{Bus, Core};

pub trait AddressMode {
    const NAME: &'static str;
    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16;
}

pub struct Immediate;

impl AddressMode for Immediate {
    const NAME: &'static str = "#i";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        let address = core.pc;
        core.pc = core.pc.wrapping_add(1);
        address
    }
}

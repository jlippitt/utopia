use super::{Bus, Core};

pub trait AddressMode {
    const NAME: &'static str;
    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16;
}

fn add_index(core: &mut Core<impl Bus>, base: u16, index: u8, write: bool) -> u16 {
    let indexed = base.wrapping_add(index as u16);

    if write || (indexed & 0xff00) != (base & 0xff00) {
        core.read((base & 0xff00) | (indexed & 0xff));
    }

    indexed
}

pub struct Immediate;

impl AddressMode for Immediate {
    const NAME: &'static str = "#const";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        let address = core.pc;
        core.pc = core.pc.wrapping_add(1);
        address
    }
}

pub struct Absolute;

impl AddressMode for Absolute {
    const NAME: &'static str = "addr";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        core.next_word()
    }
}

pub struct AbsoluteX;

impl AddressMode for AbsoluteX {
    const NAME: &'static str = "addr,X";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16 {
        let base = core.next_word();
        add_index(core, base, core.x, write)
    }
}

pub struct AbsoluteY;

impl AddressMode for AbsoluteY {
    const NAME: &'static str = "addr,Y";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16 {
        let base = core.next_word();
        add_index(core, base, core.y, write)
    }
}

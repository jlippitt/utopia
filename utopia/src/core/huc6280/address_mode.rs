use super::{Bus, Core, ZERO_PAGE};

pub trait AddressMode {
    const NAME: &'static str;
    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32;
}

fn add_index(core: &mut Core<impl Bus>, base: u16, index: u8, write: bool) -> u16 {
    let indexed = base.wrapping_add(index as u16);

    if write || (indexed & 0xff00) != (base & 0xff00) {
        core.read((base & 0xff00) | (indexed & 0xff));
    }

    indexed
}

fn get_indirect(core: &mut Core<impl Bus>, direct: u8) -> u16 {
    let low = core.read_physical(ZERO_PAGE | direct as u32);
    let high = core.read_physical(ZERO_PAGE | direct.wrapping_add(1) as u32);
    u16::from_le_bytes([low, high])
}

pub struct Immediate;

impl AddressMode for Immediate {
    const NAME: &'static str = "#const";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        let address = core.pc;
        core.pc = core.pc.wrapping_add(1);
        core.map(address)
    }
}

pub struct Absolute;

impl AddressMode for Absolute {
    const NAME: &'static str = "addr";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        let base = core.next_word();
        core.map(base)
    }
}

pub struct AbsoluteX;

impl AddressMode for AbsoluteX {
    const NAME: &'static str = "addr,X";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = core.next_word();
        let indexed = add_index(core, base, core.x, write);
        core.map(indexed)
    }
}

pub struct AbsoluteY;

impl AddressMode for AbsoluteY {
    const NAME: &'static str = "addr,Y";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = core.next_word();
        let indexed = add_index(core, base, core.y, write);
        core.map(indexed)
    }
}

pub struct ZeroPage;

impl AddressMode for ZeroPage {
    const NAME: &'static str = "zp";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        ZERO_PAGE | core.next_byte() as u32
    }
}

pub struct ZeroPageX;

impl AddressMode for ZeroPageX {
    const NAME: &'static str = "zp,X";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        let base = core.next_byte();
        core.read_physical(ZERO_PAGE | base as u32);
        ZERO_PAGE | base.wrapping_add(core.x) as u32
    }
}

pub struct ZeroPageY;

impl AddressMode for ZeroPageY {
    const NAME: &'static str = "zp,Y";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        let base = core.next_byte();
        core.read_physical(ZERO_PAGE | base as u32);
        ZERO_PAGE | base.wrapping_add(core.y) as u32
    }
}

pub struct ZeroPageXIndirect;

impl AddressMode for ZeroPageXIndirect {
    const NAME: &'static str = "(zp,X)";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u32 {
        let base = core.next_byte();
        core.read_physical(ZERO_PAGE | base as u32);
        let indexed = base.wrapping_add(core.x);
        let indirect = get_indirect(core, indexed);
        core.map(indirect)
    }
}

pub struct ZeroPageIndirectY;

impl AddressMode for ZeroPageIndirectY {
    const NAME: &'static str = "(zp),Y";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let direct = core.next_byte();
        let indirect = get_indirect(core, direct);
        let indexed = add_index(core, indirect, core.y, write);
        core.map(indexed)
    }
}

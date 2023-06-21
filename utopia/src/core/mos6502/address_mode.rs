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

fn get_indirect(core: &mut Core<impl Bus>, direct: u8) -> u16 {
    let low = core.read(direct as u16);
    let high = core.read(direct.wrapping_add(1) as u16);
    u16::from_le_bytes([low, high])
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

pub struct ZeroPage;

impl AddressMode for ZeroPage {
    const NAME: &'static str = "zp";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        core.next_byte() as u16
    }
}

pub struct ZeroPageX;

impl AddressMode for ZeroPageX {
    const NAME: &'static str = "zp,X";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        let base = core.next_byte();
        core.read(base as u16);
        base.wrapping_add(core.x) as u16
    }
}

pub struct ZeroPageY;

impl AddressMode for ZeroPageY {
    const NAME: &'static str = "zp,Y";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        let base = core.next_byte();
        core.read(base as u16);
        base.wrapping_add(core.y) as u16
    }
}

pub struct ZeroPageXIndirect;

impl AddressMode for ZeroPageXIndirect {
    const NAME: &'static str = "(zp,X)";

    fn resolve(core: &mut Core<impl Bus>, _write: bool) -> u16 {
        let base = core.next_byte();
        core.read(base as u16);
        let direct = base.wrapping_add(core.x);
        get_indirect(core, direct)
    }
}

pub struct ZeroPageIndirectY;

impl AddressMode for ZeroPageIndirectY {
    const NAME: &'static str = "(zp),Y";

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u16 {
        let direct = core.next_byte();
        let indirect = get_indirect(core, direct);
        add_index(core, indirect, core.y, write)
    }
}

use super::{Bus, Core};

const WRAP16: u32 = 0x0000_ffff;
const WRAP24: u32 = 0x00ff_ffff;

fn index_absolute<const X: bool>(
    core: &mut Core<impl Bus>,
    base: u32,
    index: u16,
    write: bool,
) -> u32 {
    let address = base.wrapping_add(index as u32) & WRAP24;

    if !X || write || (address & 0xffff_ff00) != (base & 0xffff_ff00) {
        core.idle();
    }

    address
}

fn index_direct<const E: bool>(core: &Core<impl Bus>, base: u32, index: u16) -> u32 {
    if E && (core.d & 0xff) == 0 {
        (base & 0xff00) | (base.wrapping_add(index as u32) & 0xff)
    } else {
        base.wrapping_add(index as u32) & WRAP16
    }
}

fn get_indirect<const E: bool>(core: &mut Core<impl Bus>, low_address: u32) -> u32 {
    let low = core.read(low_address);
    let high_address = index_direct::<E>(core, low_address, 1);
    let high = core.read(high_address);
    let target = u16::from_le_bytes([low, high]);
    core.dbr | (target as u32)
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

pub struct AbsoluteX<const X: bool>;

impl<const X: bool> AddressMode for AbsoluteX<X> {
    const NAME: &'static str = "addr,X";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = Absolute::resolve(core, write);
        index_absolute::<X>(core, base, core.x, write)
    }
}

pub struct AbsoluteY<const X: bool>;

impl<const X: bool> AddressMode for AbsoluteY<X> {
    const NAME: &'static str = "addr,Y";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = Absolute::resolve(core, write);
        index_absolute::<X>(core, base, core.y, write)
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
        index_direct::<E>(core, base, core.x)
    }
}

pub struct DirectY<const E: bool>;

impl<const E: bool> AddressMode for DirectY<E> {
    const NAME: &'static str = "dp,Y";
    const WRAP: u32 = WRAP16;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = Direct::resolve(core, write);
        core.idle();
        index_direct::<E>(core, base, core.y)
    }
}

pub struct DirectIndirect<const E: bool>;

impl<const E: bool> AddressMode for DirectIndirect<E> {
    const NAME: &'static str = "(dp)";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let low_address = Direct::resolve(core, write);
        get_indirect::<E>(core, low_address)
    }
}

pub struct DirectXIndirect<const E: bool>;

impl<const E: bool> AddressMode for DirectXIndirect<E> {
    const NAME: &'static str = "(dp,X)";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let low_address = DirectX::<E>::resolve(core, write);
        get_indirect::<E>(core, low_address)
    }
}

pub struct DirectIndirectY<const E: bool, const X: bool>;

impl<const E: bool, const X: bool> AddressMode for DirectIndirectY<E, X> {
    const NAME: &'static str = "(dp),Y";
    const WRAP: u32 = WRAP24;

    fn resolve(core: &mut Core<impl Bus>, write: bool) -> u32 {
        let base = DirectIndirect::<E>::resolve(core, write);
        index_absolute::<X>(core, base, core.y, write)
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

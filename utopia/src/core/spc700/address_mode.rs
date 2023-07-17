use super::{Bus, Core};

pub trait ReadAddress {
    const NAME: &'static str;
    fn read(core: &mut Core<impl Bus>) -> u8;
}

pub trait WriteAddress: ReadAddress {
    fn write(core: &mut Core<impl Bus>, value: u8);
}

pub struct X;

impl ReadAddress for X {
    const NAME: &'static str = "X";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.x
    }
}

impl WriteAddress for X {
    fn write(core: &mut Core<impl Bus>, value: u8) {
        core.x = value;
    }
}

pub struct Immediate;

impl ReadAddress for Immediate {
    const NAME: &'static str = "#i";

    fn read(core: &mut Core<impl Bus>) -> u8 {
        core.next_byte()
    }
}

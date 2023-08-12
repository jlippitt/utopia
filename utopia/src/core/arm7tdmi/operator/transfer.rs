use super::super::{Bus, Core};

pub trait TransferOperator {
    const NAME: &'static str;
    fn apply<const SIZE: usize>(core: &mut Core<impl Bus>, rd: usize, address: u32);
}

pub struct Str;

impl TransferOperator for Str {
    const NAME: &'static str = "STR";

    fn apply<const SIZE: usize>(core: &mut Core<impl Bus>, rd: usize, address: u32) {
        match SIZE {
            0 => core.write_byte(address, core.get(rd) as u8),
            1 => core.write_halfword(address, core.get(rd) as u16),
            2 => core.write_word(address, core.get(rd)),
            _ => unreachable!(),
        }
    }
}

pub struct Ldr;

impl TransferOperator for Ldr {
    const NAME: &'static str = "LDR";

    fn apply<const SIZE: usize>(core: &mut Core<impl Bus>, rd: usize, address: u32) {
        let result = match SIZE {
            0 => core.read_byte(address) as u32,
            1 => core.read_halfword(address) as u32,
            2 => core.read_word(address),
            _ => unreachable!(),
        };

        core.set(rd, result);
    }
}

pub struct Lds;

impl TransferOperator for Lds {
    const NAME: &'static str = "LDS";

    fn apply<const SIZE: usize>(core: &mut Core<impl Bus>, rd: usize, address: u32) {
        let result = match SIZE {
            0 => core.read_byte(address) as i8 as i32 as u32,
            1 => core.read_halfword(address) as i16 as i32 as u32,
            2 => core.read_word(address),
            _ => unreachable!(),
        };

        core.set(rd, result);
    }
}

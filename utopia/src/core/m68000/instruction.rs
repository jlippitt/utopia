use super::{Bus, Core, Mode, Size};
use address_mode::AddressMode;
use tracing::trace;

mod address_mode;

pub fn reset(core: &mut Core<impl Bus>) {
    trace!("RESET");
    core.set_mode(Mode::Supervisor);
    core.set_areg::<u32>(7, core.read(0x0000_0000));
    core.set_pc(core.read(0x0000_0004));
    core.set_int_level(7);
}

pub fn dispatch(core: &mut Core<impl Bus>) {
    let word: u16 = core.next();

    #[allow(clippy::unusual_byte_groupings)]
    match word >> 6 {
        0b0100_1010_00 => tst::<u8>(core, word),
        0b0100_1010_01 => tst::<u16>(core, word),
        0b0100_1010_10 => tst::<u32>(core, word),

        _ => unimplemented!(
            "M68000 Opcode: {:04b}_{:04b}_{:02b}",
            (word >> 12) & 15,
            (word >> 8) & 15,
            (word >> 6) & 3
        ),
    }
}

fn tst<T: Size>(core: &mut Core<impl Bus>, word: u16) {
    let operand = AddressMode::from(word);
    trace!("TST.{} {}", T::NAME, operand);
    let value: T = operand.read(core);
    core.set_ccr(|flags| {
        flags.set_nz(value);
        flags.v = 0;
        flags.c = false;
    });
}

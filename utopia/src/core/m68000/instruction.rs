use super::{Bus, Core, Mode, Size};
use address_mode::AddressMode;
use tracing::trace;

mod address_mode;
mod condition;
mod control;

pub fn reset(core: &mut Core<impl Bus>) {
    trace!("RESET");
    core.set_mode(Mode::Supervisor);
    core.set_areg::<u32>(7, core.read(0x0000_0000));
    core.set_pc(core.read(0x0000_0004));
    core.set_int_level(7);
}

pub fn dispatch(core: &mut Core<impl Bus>) {
    use condition as cond;

    let word: u16 = core.next();

    #[allow(clippy::unusual_byte_groupings)]
    match word >> 6 {
        0b0100_1010_00 => tst::<u8>(core, word),
        0b0100_1010_01 => tst::<u16>(core, word),
        0b0100_1010_10 => tst::<u32>(core, word),

        0b0110_0000_00..=0b0110_0000_11 => control::bra(core, word),
        //0b0110_0001_00..=0b0110_0001_11 => control::bsr(core, word),
        0b0110_0010_00..=0b0110_0010_11 => control::bcc::<cond::HI>(core, word),
        0b0110_0011_00..=0b0110_0011_11 => control::bcc::<cond::LS>(core, word),
        0b0110_0100_00..=0b0110_0100_11 => control::bcc::<cond::CC>(core, word),
        0b0110_0101_00..=0b0110_0101_11 => control::bcc::<cond::CS>(core, word),
        0b0110_0110_00..=0b0110_0110_11 => control::bcc::<cond::NE>(core, word),
        0b0110_0111_00..=0b0110_0111_11 => control::bcc::<cond::EQ>(core, word),
        0b0110_1000_00..=0b0110_1000_11 => control::bcc::<cond::VC>(core, word),
        0b0110_1001_00..=0b0110_1001_11 => control::bcc::<cond::VS>(core, word),
        0b0110_1010_00..=0b0110_1010_11 => control::bcc::<cond::PL>(core, word),
        0b0110_1011_00..=0b0110_1011_11 => control::bcc::<cond::MI>(core, word),
        0b0110_1100_00..=0b0110_1100_11 => control::bcc::<cond::GE>(core, word),
        0b0110_1101_00..=0b0110_1101_11 => control::bcc::<cond::LT>(core, word),
        0b0110_1110_00..=0b0110_1110_11 => control::bcc::<cond::GT>(core, word),
        0b0110_1111_00..=0b0110_1111_11 => control::bcc::<cond::LE>(core, word),

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

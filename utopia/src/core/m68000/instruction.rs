use super::{Bus, Core, Mode};
use tracing::trace;

pub fn reset(core: &mut Core<impl Bus>) {
    trace!("RESET");
    core.set_mode(Mode::Supervisor);
    core.set_areg::<u32>(7, core.read(0x0000_0000));
    core.set_pc(core.read(0x0000_0004));
    core.set_int_level(7);
}

pub fn dispatch(core: &mut Core<impl Bus>) {
    let word: u16 = core.next();

    match word >> 10 {
        _ => unimplemented!(
            "M68000 Opcode: {:04b}_{:04b}_{:02b}",
            (word >> 12) & 15,
            (word >> 8) & 15,
            (word >> 6) & 3
        ),
    }
}

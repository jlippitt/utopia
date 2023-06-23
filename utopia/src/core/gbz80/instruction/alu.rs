use super::super::{Core, Bus, ReadAddress, WriteAddress};
use tracing::debug;

pub fn xor<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("XOR A, {}", Rhs::NAME);
    core.a ^= Rhs::read(core);
    core.flags.z = core.a;
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}

pub fn inc<Addr: WriteAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("INC {}", Addr::NAME);
    let result = Addr::read(core).wrapping_add(1);
    Addr::write(core, result);
    core.flags.z = result;
    core.flags.n = false;
    core.flags.h = (result & 0x0f) == 0;
}
use super::super::{Core, Bus, ReadAddress};
use tracing::debug;

pub fn xor<Rhs: ReadAddress<u8>>(core: &mut Core<impl Bus>) {
    debug!("XOR A, {}", Rhs::NAME);
    core.a ^= Rhs::read(core);
    core.flags.z = core.a;
    core.flags.n = false;
    core.flags.h = false;
    core.flags.c = false;
}
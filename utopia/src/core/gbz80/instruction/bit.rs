use super::super::{Core, Bus, ReadAddress};
use tracing::debug;

fn bit_from_opcode(opcode: u8) -> u8 {
   (opcode >> 3) & 7
}

pub fn bit<Addr: ReadAddress<u8>>(core: &mut Core<impl Bus>, opcode: u8) {
    let bit = bit_from_opcode(opcode);
    debug!("BIT {}, {}", bit, Addr::NAME);
    core.flags.z = Addr::read(core) & (1 << bit);
    core.flags.n = false;
    core.flags.h = true;
}
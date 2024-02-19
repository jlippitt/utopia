use super::{AddressMode, Bus, Core, Operator, Size};
use tracing::trace;

pub fn immediate<T: Operator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);

    if dst.is_immediate() {
        todo!("Bitwise CCR operations");
    }

    trace!("{}I.{} #const, {}", T::NAME, U::NAME, dst);
    let src = core.next();
    dst.modify(core, |core, value| T::apply::<U>(core, value, src))
}

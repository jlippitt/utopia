use super::{AddressMode, Bus, Core, Operator, Size};
use tracing::trace;

pub fn immediate<T: Operator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);

    if dst.is_immediate() {
        todo!("Bitwise CCR operations");
    }

    trace!("{}I.{} #imm, {}", T::NAME, U::NAME, dst);
    let src = core.next();
    dst.modify(core, |core, value| T::apply::<U>(core, value, src));
}

pub fn read<T: Operator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    let dst = (word >> 9) & 7;
    trace!("{}.{} {}, D{}", T::NAME, U::NAME, src, dst);
    let src_value: U = src.read(core);
    let dst_value = core.dreg(dst as usize);
    let result = T::apply(core, dst_value, src_value);
    core.set_dreg(dst as usize, result);
}

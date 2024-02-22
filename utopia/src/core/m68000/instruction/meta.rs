use super::operator::{BinaryOperator, UnaryOperator};
use super::{AddressMode, Bus, Core, Size};
use tracing::trace;

pub fn immediate<T: BinaryOperator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);

    if dst.is_immediate() {
        todo!("Bitwise CCR operations");
    }

    trace!("{}I.{} #imm, {}", T::NAME, U::NAME, dst);
    let src = core.next();
    dst.modify(core, |core, value| T::apply::<U>(core, value, src));
}

pub fn quick<T: BinaryOperator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);
    let src = ((((word >> 9) - 1) & 7) + 1) as u8;
    trace!("{}Q.{} #{}, {}", T::NAME, U::NAME, src, dst);
    dst.modify(core, |core, value| {
        T::apply::<U>(core, value, U::from(src).unwrap())
    });
}

pub fn read<T: BinaryOperator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let src = AddressMode::from(word);
    let dst = (word >> 9) & 7;
    trace!("{}.{} {}, D{}", T::NAME, U::NAME, src, dst);
    let src_value: U = src.read(core);
    let dst_value = core.dreg(dst as usize);
    let result = T::apply(core, dst_value, src_value);
    core.set_dreg(dst as usize, result);
}

pub fn unary<T: UnaryOperator, U: Size>(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);
    trace!("{}.{} {}", T::NAME, U::NAME, dst);
    dst.modify(core, T::apply::<U>);
}

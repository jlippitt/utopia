use super::super::address_mode::AddressMode;
use super::super::operator::{ReadOperator, WriteOperator};
use super::super::{Bus, Core};
use tracing::debug;

pub fn read<Addr: AddressMode, Op: ReadOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    let address = Addr::resolve(core, false);
    let value = core.read(address);
    Op::apply(core, value);
}

pub fn write<Addr: AddressMode, Op: WriteOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    let address = Addr::resolve(core, true);
    let value = Op::apply(core);
    core.write(address, value);
}

use super::super::address_mode::AddressMode;
use super::super::operator::WriteOperator;
use super::super::{Bus, Core};
use tracing::debug;

pub fn write<const MX: bool, Addr: AddressMode, Op: WriteOperator>(core: &mut Core<impl Bus>) {
    if MX {
        debug!("{}.B {}", Op::NAME, Addr::NAME);
        let address = Addr::resolve(core, true);
        core.poll();
        let value = Op::apply8(core);
        core.write(address, value);
    } else {
        debug!("{}.W {}", Op::NAME, Addr::NAME);
        let address_low = Addr::resolve(core, true);
        let value = Op::apply16(core);
        core.write(address_low, value as u8);
        core.poll();
        let address_high = address_low.wrapping_add(1) & Addr::WRAP;
        core.write(address_high, (value >> 8) as u8);
    }
}

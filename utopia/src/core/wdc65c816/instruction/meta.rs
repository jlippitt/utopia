use super::super::address_mode::AddressMode;
use super::super::operator::{BranchOperator, ModifyOperator, ReadOperator, WriteOperator};
use super::super::{Bus, Core};
use tracing::debug;

pub fn immediate<const MX: bool, Op: ReadOperator>(core: &mut Core<impl Bus>) {
    debug!("{}.{} #const", Op::NAME, super::size(MX));

    if MX {
        core.poll();
        let value = core.next_byte();
        Op::apply8(core, value);
    } else {
        let low = core.next_byte();
        core.poll();
        let high = core.next_byte();
        Op::apply16(core, u16::from_le_bytes([low, high]));
    }
}

pub fn read<const MX: bool, Addr: AddressMode, Op: ReadOperator>(core: &mut Core<impl Bus>) {
    debug!("{}.{} {}", Op::NAME, super::size(MX), Addr::NAME);
    let low_address = Addr::resolve(core, false);

    if MX {
        core.poll();
        let value = core.read(low_address);
        Op::apply8(core, value);
    } else {
        let low = core.read(low_address);
        core.poll();
        let high_address = low_address.wrapping_add(1) & Addr::WRAP;
        let high = core.read(high_address);
        Op::apply16(core, u16::from_le_bytes([low, high]));
    }
}

pub fn write<const MX: bool, Addr: AddressMode, Op: WriteOperator>(core: &mut Core<impl Bus>) {
    debug!("{}.{} {}", Op::NAME, super::size(MX), Addr::NAME);
    let low_address = Addr::resolve(core, true);

    if MX {
        core.poll();
        let value = Op::apply8(core);
        core.write(low_address, value);
    } else {
        let value = Op::apply16(core);
        core.write(low_address, value as u8);
        core.poll();
        let high_address = low_address.wrapping_add(1) & Addr::WRAP;
        core.write(high_address, (value >> 8) as u8);
    }
}

pub fn accumulator<const M: bool, Op: ModifyOperator>(core: &mut Core<impl Bus>) {
    debug!("{}.{} A", Op::NAME, super::size(M));
    core.poll();
    core.idle();

    if M {
        let result = Op::apply8(core, core.a as u8);
        core.a = (core.a & 0xff00) | (result as u16);
    } else {
        core.a = Op::apply16(core, core.a);
    }
}

pub fn modify<const M: bool, Addr: AddressMode, Op: ModifyOperator>(core: &mut Core<impl Bus>) {
    debug!("{}.{} {}", Op::NAME, super::size(M), Addr::NAME);
    let low_address = Addr::resolve(core, true);

    if M {
        let value = core.read(low_address);
        core.idle();
        core.poll();
        let result = Op::apply8(core, value);
        core.write(low_address, result);
    } else {
        let low_value = core.read(low_address);
        let high_address = low_address.wrapping_add(1) & Addr::WRAP;
        let high_value = core.read(high_address);
        let value = u16::from_le_bytes([low_value, high_value]);
        core.idle();
        let result = Op::apply16(core, value);
        core.write(high_address, (result >> 8) as u8);
        core.poll();
        core.write(low_address, result as u8);
    }
}

pub fn branch<const E: bool, Op: BranchOperator>(core: &mut Core<impl Bus>) {
    debug!("{} nearlabel", Op::NAME);

    if Op::apply(&core.flags) {
        let offset = ((core.next_byte() as i8) as i32) as u32;
        debug!("Branch taken");
        let target = (core.pc & 0xffff0000) | (core.pc.wrapping_add(offset) & 0xffff);

        if E && (target & 0xff00) != (core.pc & 0xff00) {
            core.idle();
        }

        core.poll();
        core.idle();
        core.pc = target;
    } else {
        core.poll();
        core.next_byte();
        debug!("Branch not taken");
    }
}

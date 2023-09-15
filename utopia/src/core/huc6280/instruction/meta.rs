use super::super::address_mode::AddressMode;
use super::super::operator::{BranchOperator, ModifyOperator, ReadOperator, WriteOperator};
use super::super::{Bus, Core};
use tracing::debug;

pub fn read<Addr: AddressMode, Op: ReadOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    let address = Addr::resolve(core, false);
    core.poll();
    let value = core.read_physical(address);
    Op::apply(core, value);
}

pub fn write<Addr: AddressMode, Op: WriteOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    let address = Addr::resolve(core, true);
    core.poll();
    let value = Op::apply(core);
    core.write_physical(address, value);
}

pub fn accumulator<Op: ModifyOperator>(core: &mut Core<impl Bus>) {
    debug!("{} A", Op::NAME);
    core.poll();
    core.read(core.pc);
    core.a = Op::apply(core, core.a);
}

pub fn modify<Addr: AddressMode, Op: ModifyOperator>(core: &mut Core<impl Bus>) {
    debug!("{} {}", Op::NAME, Addr::NAME);
    let address = Addr::resolve(core, true);
    let input = core.read_physical(address);
    core.read_physical(address);
    core.poll();
    let result = Op::apply(core, input);
    core.write_physical(address, result);
}

pub fn branch<Op: BranchOperator>(core: &mut Core<impl Bus>) {
    debug!("{} nearlabel", Op::NAME);

    if Op::apply(core) {
        debug!("Branch taken");
        let offset = core.next_byte() as i8;
        let target = ((core.pc as i16).wrapping_add(offset as i16)) as u16;

        if (target & 0xff00) != (core.pc & 0xff00) {
            core.read(core.pc);
            core.poll();
            core.read((core.pc & 0xff00) | (target & 0xff));
        } else {
            core.poll();
            core.read(core.pc);
        }

        core.pc = target;
    } else {
        debug!("Branch not taken");
        core.poll();
        core.next_byte();
    }
}

use super::Cp2;
use crate::system::n64::mips::{Bus, Core, GPR};
use bitfield_struct::bitfield;
use tracing::trace;

#[bitfield(u32)]
struct LoadStore {
    #[bits(7)]
    offset: u32,
    #[bits(4)]
    element: usize,
    #[bits(5)]
    opcode: u32,
    #[bits(5)]
    vt: usize,
    #[bits(5)]
    base: usize,
    #[bits(6)]
    __: u32,
}

pub fn lbv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LBV", core, word, 0);
    let value = core.read_u8(address);
    core.cp2_mut().set_u8(op.vt(), op.element(), value);
}

pub fn lsv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LSV", core, word, 1);
    let value = core.read_u16(address);
    core.cp2_mut().set_u16(op.vt(), op.element(), value);
}

pub fn llv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LLV", core, word, 2);
    let value = core.read_u32(address);
    core.cp2_mut().set_u32(op.vt(), op.element(), value);
}

pub fn ldv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LDV", core, word, 3);
    let value = core.read_u64(address);
    core.cp2_mut().set_u64(op.vt(), op.element(), value);
}

pub fn lqv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LQV", core, word, 4);

    if (address & 15) == 0 && op.element() == 0 {
        let high = core.read_u64(address) as u128;
        let low = core.read_u64(address.wrapping_add(8)) as u128;
        let value = (high << 64) | low;
        core.cp2_mut().set_u128(op.vt(), value);
        return;
    }

    let end = (address + 0x10) & !0x0f;
    let size = (end - address).min(16 - op.element() as u32);
    let mut reg = core.cp2().regs[op.vt()];

    for index in 0..size {
        reg.write(op.element() + index as usize, core.read_u8(address + index));
    }

    core.cp2_mut().set_le(op.vt(), reg.to_le_array());
}

pub fn lrv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, end) = decode("LRV", core, word, 4);
    let address = end & !0x0f;
    let size = (end - address).min(16 - op.element() as u32);
    let offset = op.element() + (16 - size as usize);
    let mut reg = core.cp2().regs[op.vt()];

    for index in 0..size {
        reg.write(offset + index as usize, core.read_u8(address + index));
    }

    core.cp2_mut().set_le(op.vt(), reg.to_le_array());
}

pub fn lpv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LPV", core, word, 0);

    let values = std::array::from_fn(|index| {
        let byte = core.read_u8(address.wrapping_add(index as u32));
        (byte as u16) << 8
    });

    core.cp2_mut().set_be(op.vt(), values);
}

pub fn luv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LUV", core, word, 0);

    let values = std::array::from_fn(|index| {
        let byte = core.read_u8(address.wrapping_add(index as u32));
        (byte as u16) << 7
    });

    core.cp2_mut().set_be(op.vt(), values);
}

pub fn ltv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("LTV", core, word, 4);

    let reg = op.vt() & 0x18;
    let element = (8 - (op.element() >> 1)) & 7;

    for index in 0..8 {
        let value = core.read_u16(address.wrapping_add((index << 1) as u32));
        core.cp2_mut()
            .set_lane(reg + index, (element + index) & 7, value);
    }
}

pub fn sbv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SBV", core, word, 0);
    let value = core.cp2().get_u8(op.vt(), op.element());
    core.write_u8(address, value);
}

pub fn ssv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SSV", core, word, 1);
    let value = core.cp2().get_u16(op.vt(), op.element());
    core.write_u16(address, value);
}

pub fn slv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SLV", core, word, 2);
    let value = core.cp2().get_u32(op.vt(), op.element());
    core.write_u32(address, value);
}

pub fn sdv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SDV", core, word, 3);
    let value = core.cp2().get_u64(op.vt(), op.element());
    core.write_u64(address, value);
}

pub fn sqv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SQV", core, word, 4);

    if (address & 15) == 0 && op.element() == 0 {
        let value = core.cp2().get_u128(op.vt());
        core.write_u64(address, (value >> 64) as u64);
        core.write_u64(address.wrapping_add(8), value as u64);
        return;
    }

    let end = (address + 0x10) & !0x0f;
    let size = (end - address).min(16 - op.element() as u32);
    let reg = core.cp2().regs[op.vt()];

    for index in 0..size {
        core.write_u8(address + index, reg.read(op.element() + index as usize));
    }
}

pub fn srv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, end) = decode("SRV", core, word, 4);
    let address = end & !0x0f;
    let size = (end - address).min(16 - op.element() as u32);
    let offset = op.element() + (16 - size as usize);
    let reg = core.cp2().regs[op.vt()];

    for index in 0..size {
        core.write_u8(address + index, reg.read(offset + index as usize));
    }
}

pub fn spv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SPV", core, word, 0);

    let values = core.cp2().get_be(op.vt());

    for (index, value) in values.iter().enumerate() {
        let byte = (value >> 8) as u8;
        core.write_u8(address.wrapping_add(index as u32), byte);
    }

    core.cp2_mut().set_be(op.vt(), values);
}

pub fn suv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("SUV", core, word, 0);

    let values = core.cp2().get_be(op.vt());

    for (index, value) in values.iter().enumerate() {
        let byte = (value >> 7) as u8;
        core.write_u8(address.wrapping_add(index as u32), byte);
    }

    core.cp2_mut().set_be(op.vt(), values);
}

pub fn stv(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    let (op, address) = decode("STV", core, word, 4);

    let reg = op.vt() & 0x18;
    let element = op.element() >> 1;

    for index in 0..8 {
        let value = core.cp2().lane(reg + ((element + index) & 7), index);
        core.write_u16(address.wrapping_add((index << 1) as u32), value);
    }
}

fn decode(
    name: &'static str,
    core: &mut Core<impl Bus<Cp2 = Cp2>>,
    word: u32,
    shift: u32,
) -> (LoadStore, u32) {
    let op = LoadStore::from(word);
    let offset = (((op.offset() << 25) as i32) >> 25) << shift;

    trace!(
        "{:08X} {} V{:02}[{}], {}({})",
        core.pc(),
        name,
        op.vt(),
        op.element(),
        offset,
        GPR[op.base()]
    );

    let address = core.getw(op.base()).wrapping_add(offset as u32);

    (op, address)
}

use super::super::{Bus, Core, REGS, SIZES};
use tracing::debug;

pub fn ldr_immediate<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = ((word >> 6) & 31) << SIZE;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDR{} {}, [{}, #0x{:X}]",
        pc, SIZES[SIZE], REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);

    let result = match SIZE {
        0 => core.read_byte(address) as u32,
        1 => core.read_halfword(address) as u32,
        2 => core.read_word(address),
        _ => unreachable!(),
    };

    core.set(rd, result);
}

pub fn str_immediate<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = ((word >> 6) & 31) << SIZE;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} STR{} {}, [{}, #0x{:X}]",
        pc, SIZES[SIZE], REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);

    match SIZE {
        0 => core.write_byte(address, core.get(rd) as u8),
        1 => core.write_halfword(address, core.get(rd) as u16),
        2 => core.write_word(address, core.get(rd)),
        _ => unreachable!(),
    }
}

pub fn ldr_register<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let ro = ((word >> 6) & 7) as usize;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDR{} {}, [{}, {}]",
        pc, SIZES[SIZE], REGS[rd], REGS[rb], REGS[ro]
    );

    let address = core.get(rb).wrapping_add(core.get(ro));

    let result = match SIZE {
        0 => core.read_byte(address) as u32,
        1 => core.read_halfword(address) as u32,
        2 => core.read_word(address),
        _ => unreachable!(),
    };

    core.set(rd, result);
}

pub fn str_register<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let ro = ((word >> 6) & 7) as usize;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} STR{} {}, [{}, {}]",
        pc, SIZES[SIZE], REGS[rd], REGS[rb], REGS[ro]
    );

    let address = core.get(rb).wrapping_add(core.get(ro));

    match SIZE {
        0 => core.write_byte(address, core.get(rd) as u8),
        1 => core.write_halfword(address, core.get(rd) as u16),
        2 => core.write_word(address, core.get(rd)),
        _ => unreachable!(),
    }
}

pub fn lds_register<const SIZE: usize>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let ro = ((word >> 6) & 7) as usize;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDS{} {}, [{}, {}]",
        pc, SIZES[SIZE], REGS[rd], REGS[rb], REGS[ro]
    );

    let address = core.get(rb).wrapping_add(core.get(ro));

    let result = match SIZE {
        0 => core.read_byte(address) as i8 as i32 as u32,
        1 => core.read_halfword(address) as i16 as i32 as u32,
        _ => unreachable!(),
    };

    core.set(rd, result);
}

pub fn ldr_pc_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    debug!("{:08X} LDR {}, [PC, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.pc.wrapping_add(2).wrapping_add(offset as u32) & 0xffff_fffd;
    let result = core.read_word(address);
    core.set(rd, result);
}

pub fn ldr_sp_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    debug!("{:08X} LDR {}, [SP, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.regs[13].wrapping_add(offset as u32);
    let result = core.read_word(address);
    core.set(rd, result);
}

pub fn str_sp_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    debug!("{:08X} STR {}, [SP, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.regs[13].wrapping_add(offset as u32);
    core.write_word(address, core.get(rd));
}

pub fn load_address<const SP: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    debug!(
        "{:08X} ADD {}, {}, #0x{:X}",
        pc,
        REGS[rd],
        if SP { "SP" } else { "PC" },
        offset
    );

    let address = if SP {
        core.regs[13].wrapping_add(offset as u32)
    } else {
        core.pc.wrapping_add(2).wrapping_add(offset as u32) & 0xffff_fffd
    };

    core.set(rd, address);
}

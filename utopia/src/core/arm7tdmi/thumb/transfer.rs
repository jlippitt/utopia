use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn ldr_byte(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = (word >> 6) & 31;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDRB {}, [{}, #0x{:X}]",
        pc, REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    let result = core.read_byte(address);
    core.set(rd, result as u32);
}

pub fn ldr_halfword(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = ((word >> 6) & 31) << 1;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDRH {}, [{}, #0x{:X}]",
        pc, REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    let result = core.read_halfword(address);
    core.set(rd, result as u32);
}

pub fn ldr_word(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = ((word >> 6) & 31) << 2;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDR {}, [{}, #0x{:X}]",
        pc, REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    let result = core.read_word(address);
    core.set(rd, result);
}

pub fn str_byte(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = (word >> 6) & 31;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} STRB {}, [{}, #0x{:X}]",
        pc, REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    core.write_byte(address, core.get(rd) as u8);
}

pub fn str_halfword(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = ((word >> 6) & 31) << 1;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} STRH {}, [{}, #0x{:X}]",
        pc, REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    core.write_halfword(address, core.get(rd) as u16);
}

pub fn str_word(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let offset = ((word >> 6) & 31) << 2;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} STR {}, [{}, #0x{:X}]",
        pc, REGS[rd], REGS[rb], offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    core.write_word(address, core.get(rd));
}

pub fn ldr_register<const BYTE: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let ro = ((word >> 6) & 7) as usize;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} LDR{} {}, [{}, {}]",
        pc,
        if BYTE { "B " } else { "" },
        REGS[rd],
        REGS[rb],
        REGS[ro]
    );

    let address = core.get(rb).wrapping_add(core.get(ro));

    let result = if BYTE {
        core.read_byte(address) as u32
    } else {
        core.read_word(address)
    };

    core.set(rd, result);
}

pub fn str_register<const BYTE: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let ro = ((word >> 6) & 7) as usize;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    debug!(
        "{:08X} STR{} {}, [{}, {}]",
        pc,
        if BYTE { "B " } else { "" },
        REGS[rd],
        REGS[rb],
        REGS[ro]
    );

    let address = core.get(rb).wrapping_add(core.get(ro));

    if BYTE {
        core.write_byte(address, core.get(rd) as u8);
    } else {
        core.write_word(address, core.get(rd));
    }
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
        "{:08X} ADD {}, [{}, #0x{:X}]",
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

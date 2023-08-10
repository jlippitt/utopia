use super::super::{Bus, Core, REGS};
use arrayvec::ArrayVec;
use tracing::debug;

fn reg_list(word: u16, extra: Option<usize>) -> String {
    let mut reg_list: ArrayVec<&str, 9> = ArrayVec::new();

    for reg in 0..=7 {
        let mask = 1 << reg;

        if (word & mask) != 0 {
            reg_list.push(REGS[reg]);
        }
    }

    if let Some(reg) = extra {
        reg_list.push(REGS[reg]);
    }

    reg_list.join(", ")
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

pub fn ldr_pc_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    debug!("{:08X} LDR {}, [PC, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.pc.wrapping_add(offset as u32);
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

pub fn pop<const PC: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    debug!(
        "{:08X} POP {{ {} }}",
        pc,
        reg_list(word, if PC { Some(15) } else { None })
    );

    for reg in 0..=7 {
        let mask = 1 << reg;

        if (word & mask) != 0 {
            let result = core.read_word(core.regs[13]);
            core.set(reg, result);
            core.regs[13] = core.regs[13].wrapping_add(4);
        }
    }

    if PC {
        let result = core.read_word(core.regs[13]);
        core.set(15, result);
        core.regs[13] = core.regs[13].wrapping_add(4);
    }

    debug!("  {}: {:08X}", REGS[13], core.regs[13]);
}

pub fn push<const LR: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    debug!(
        "{:08X} PUSH {{ {} }}",
        pc,
        reg_list(word, if LR { Some(14) } else { None })
    );

    if LR {
        core.regs[13] = core.regs[13].wrapping_sub(4);
        core.write_word(core.regs[13], core.get(14));
    }

    for reg in (0..=7).rev() {
        let mask = 1 << reg;

        if (word & mask) != 0 {
            core.regs[13] = core.regs[13].wrapping_sub(4);
            core.write_word(core.regs[13], core.get(reg));
        }
    }

    debug!("  {}: {:08X}", REGS[13], core.regs[13]);
}

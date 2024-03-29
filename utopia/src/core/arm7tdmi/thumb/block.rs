use super::super::{Bus, Core, REGS};
use arrayvec::ArrayVec;
use tracing::trace;

fn reg_list(word: u16, extra: Option<usize>) -> String {
    let mut reg_list: ArrayVec<&str, 9> = ArrayVec::new();

    for (index, value) in REGS[0..=7].iter().enumerate() {
        let mask = 1 << index;

        if (word & mask) != 0 {
            reg_list.push(value);
        }
    }

    if let Some(reg) = extra {
        reg_list.push(REGS[reg]);
    }

    reg_list.join(", ")
}

pub fn pop<const PC: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    trace!(
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
        core.pc = result & 0xffff_fffe;
        core.regs[13] = core.regs[13].wrapping_add(4);
    }

    trace!("  {}: {:08X}", REGS[13], core.regs[13]);
}

pub fn push<const LR: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    trace!(
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

    trace!("  {}: {:08X}", REGS[13], core.regs[13]);
}

pub fn ldmia(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rb = ((word >> 8) & 7) as usize;
    let mut base = core.get(rb);

    trace!(
        "{:08X} LDMIA {}!, {{ {} }}",
        pc,
        REGS[rb],
        reg_list(word, None)
    );

    for reg in 0..=7 {
        let mask = 1 << reg;

        if (word & mask) != 0 {
            let result = core.read_word(base);
            core.set(reg, result);
            base = base.wrapping_add(4);
        }
    }

    core.set(rb, base);
}

pub fn stmia(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rb = ((word >> 8) & 7) as usize;
    let mut base = core.get(rb);

    trace!(
        "{:08X} STMIA {}!, {{ {} }}",
        pc,
        REGS[rb],
        reg_list(word, None),
    );

    for reg in (0..=7).rev() {
        let mask = 1 << reg;

        if (word & mask) != 0 {
            core.write_word(base, core.get(reg));
            base = base.wrapping_add(4);
        }
    }

    core.set(rb, base);
}

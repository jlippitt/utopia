use super::super::{Bus, Core, REGS};
use tracing::debug;

pub fn ldr_pc_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    debug!("{:08X} LDR {}, [PC, #{}]", pc, REGS[rd], offset);

    let address = core.pc.wrapping_add(offset as u32);
    let result = core.read_word(address);
    core.set(rd, result);
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

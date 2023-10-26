use super::super::operator::TransferOperator;
use super::super::{Bus, Core, REGS, SIZES};
use tracing::trace;

pub fn mem_immediate<Op: TransferOperator, const SIZE: usize>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u16,
) {
    let offset = ((word >> 6) & 31) << SIZE;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    trace!(
        "{:08X} {}{} {}, [{}, #0x{:X}]",
        pc,
        Op::NAME,
        SIZES[SIZE],
        REGS[rd],
        REGS[rb],
        offset
    );

    let address = core.get(rb).wrapping_add(offset as u32);
    Op::apply::<SIZE>(core, rd, address);
}

pub fn mem_register<Op: TransferOperator, const SIZE: usize>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u16,
) {
    let ro = ((word >> 6) & 7) as usize;
    let rb = ((word >> 3) & 7) as usize;
    let rd = (word & 7) as usize;

    trace!(
        "{:08X} {}{} {}, [{}, {}]",
        pc,
        Op::NAME,
        SIZES[SIZE],
        REGS[rd],
        REGS[rb],
        REGS[ro]
    );

    let address = core.get(rb).wrapping_add(core.get(ro));
    Op::apply::<SIZE>(core, rd, address);
}

pub fn ldr_pc_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    trace!("{:08X} LDR {}, [PC, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.pc.wrapping_add(2).wrapping_add(offset as u32) & 0xffff_fffd;
    let result = core.read_word(address);
    core.set(rd, result);
}

pub fn ldr_sp_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    trace!("{:08X} LDR {}, [SP, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.regs[13].wrapping_add(offset as u32);
    let result = core.read_word(address);
    core.set(rd, result);
}

pub fn str_sp_relative(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    trace!("{:08X} STR {}, [SP, #0x{:X}]", pc, REGS[rd], offset);

    let address = core.regs[13].wrapping_add(offset as u32);
    core.write_word(address, core.get(rd));
}

pub fn load_address<const SP: bool>(core: &mut Core<impl Bus>, pc: u32, word: u16) {
    let rd = ((word >> 8) & 7) as usize;
    let offset = (word & 0xff) << 2;

    trace!(
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

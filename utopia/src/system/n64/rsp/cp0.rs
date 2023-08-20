use crate::core::mips::{Bus, Coprocessor0, Core, REGS};
use tracing::debug;

const CREGS: [&str; 16] = [
    "DMA_CACHE",
    "DMA_DRAM",
    "DMA_READ_LENGTH",
    "DMA_WRITE_LENGTH",
    "SP_STATUS",
    "DMA_FULL",
    "DMA_BUSY",
    "SP_RESERVED",
    "CMD_START",
    "CMD_END",
    "CMD_CURRENT",
    "CMD_STATUS",
    "CMD_CLOCK",
    "CMD_BUSY",
    "CMD_PIPE_BUSY",
    "CMD_TMEM_BUSY",
];

pub struct Cp0 {
    //
}

impl Coprocessor0 for Cp0 {
    fn new() -> Self {
        Self {}
    }

    fn dispatch(core: &mut Core<impl Bus<Cp0 = Self>>, word: u32) {
        match (word >> 21) & 31 {
            0b00000 => type_r(core, mfc0, word),
            0b00100 => type_r(core, mtc0, word),
            rs => unimplemented!("RSP CP0 RS={:05b} ({:08X}: {:08X})", rs, core.pc(), word),
        }
    }
}

fn type_r<T: Bus>(core: &mut Core<T>, instr: impl Fn(&mut Core<T>, usize, usize), word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    instr(core, rt, rd);
}

fn mfc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, rt: usize, rd: usize) {
    debug!("{:08X} MFC0 {}, {}", core.pc(), REGS[rt], CREGS[rd]);

    let result = match rd {
        _ => todo!("RSP CP0 Register Read: {}", CREGS[rd]),
    };

    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, rt: usize, rd: usize) {
    debug!("{:08X} MTC0 {}, {}", core.pc(), REGS[rt], CREGS[rd]);

    let value = core.get(rt);

    match rd {
        _ => {
            if value != 0 {
                todo!("RSP CP0 Register Write: {} <= {:08X}", CREGS[rd], value);
            }
        }
    }
}

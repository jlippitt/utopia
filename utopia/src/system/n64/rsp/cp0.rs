use super::super::dma::{Dma, DmaRequest};
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
    dma_requested: Dma,
    dma_spaddr: u32,
    dma_ramaddr: u32,
    single_step: bool,
}

impl Cp0 {
    pub fn get(&self, index: usize) -> u32 {
        match index {
            4 => {
                // SP_STATUS
                // TODO
                0x01
            }
            5 => {
                // DMA_FULL
                // TODO
                0
            }
            _ => unimplemented!("RSP CP0 Register Read: {}", CREGS[index]),
        }
    }

    pub fn set(&mut self, index: usize, value: u32) -> bool {
        match index {
            0 => {
                self.dma_spaddr = value & 0x1ff8;
                debug!("SP_DMA_SPADDR: {:08X}", self.dma_spaddr);
            }
            1 => {
                self.dma_ramaddr = value & 0x00ff_fff8;
                debug!("SP_DMA_RAMADDR: {:08X}", self.dma_ramaddr);
            }
            2 => {
                self.dma_requested = Dma::Read(DmaRequest {
                    src_addr: self.dma_spaddr,
                    dst_addr: self.dma_ramaddr,
                    len: value & 0xff8f_fff8,
                })
            }
            4 => {
                // SP_STATUS
                // TODO
                if (value & 0x40) != 0 {
                    self.single_step = true;
                    debug!("RSP Single Step: {}", self.single_step);
                }

                if (value & 0x20) != 0 {
                    self.single_step = false;
                    debug!("RSP Single Step: {}", self.single_step);
                }

                if (value & 0x01) != 0 {
                    if self.single_step {
                        todo!("Single step");
                    }

                    return true;
                }
            }
            _ => unimplemented!("RSP CP0 Register Write: {} <= {:08X}", CREGS[index], value),
        }

        false
    }

    pub fn dma_requested(&self) -> Dma {
        self.dma_requested
    }

    pub fn finish_dma(&mut self) {
        self.dma_requested = Dma::None;
    }
}

impl Coprocessor0 for Cp0 {
    fn new() -> Self {
        Self {
            dma_requested: Dma::None,
            dma_spaddr: 0,
            dma_ramaddr: 0,
            single_step: false,
        }
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
    let result = core.cp0().get(rd);
    core.set(rt, result);
}

fn mtc0(core: &mut Core<impl Bus<Cp0 = Cp0>>, rt: usize, rd: usize) {
    debug!("{:08X} MTC0 {}, {}", core.pc(), REGS[rt], CREGS[rd]);
    let value = core.get(rt);
    core.cp0_mut().set(rd, value);
}

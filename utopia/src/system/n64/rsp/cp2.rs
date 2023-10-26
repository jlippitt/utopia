use crate::core::mips::{self, Bus, Core, GPR};
use bitfield_struct::bitfield;
use bitvec::array::BitArray;
use tracing::{trace, warn};
use vector::Vector;

mod compute;
mod load_store;
mod select;
mod single_lane;
mod vector;

const TABLE_SIZE: usize = 512;

const CTRL_REGS: [&str; 32] = [
    "VCO", "VCC", "VCE", "VC3", "VC4", "VC5", "VC6", "VC7", "VC8", "VC9", "VC10", "VC11", "VC12",
    "VC13", "VC14", "VC15", "VC16", "VC17", "VC18", "VC19", "VC20", "VC21", "VC22", "VC23", "VC24",
    "VC25", "VC26", "VC27", "VC28", "VC29", "VC30", "VC31",
];

pub struct Cp2 {
    regs: [Vector; 32],
    accumulator: [u64; 8],
    carry: BitArray<[u8; 1]>,
    not_equal: BitArray<[u8; 1]>,
    compare: BitArray<[u8; 1]>,
    clip_compare: BitArray<[u8; 1]>,
    compare_extension: BitArray<[u8; 1]>,
    div_in: u32,
    div_out: u32,
    reciprocal: [u16; TABLE_SIZE],
    inv_sqrt: [u16; TABLE_SIZE],
}

impl Cp2 {
    pub fn new() -> Self {
        Self {
            regs: [0.into(); 32],
            accumulator: [0; 8],
            carry: BitArray::default(),
            not_equal: BitArray::default(),
            compare: BitArray::default(),
            clip_compare: BitArray::default(),
            compare_extension: BitArray::default(),
            div_in: 0,
            div_out: 0,
            reciprocal: std::array::from_fn(reciprocal),
            inv_sqrt: std::array::from_fn(inv_sqrt),
        }
    }
}

impl Cp2 {
    fn get_u8(&self, index: usize, element: usize) -> u8 {
        self.regs[index].read(element)
    }

    fn get_u16(&self, index: usize, element: usize) -> u16 {
        self.regs[index].read(element)
    }

    fn get_u32(&self, index: usize, element: usize) -> u32 {
        self.regs[index].read(element)
    }

    fn get_u64(&self, index: usize, element: usize) -> u64 {
        self.regs[index].read(element)
    }

    fn get_u128(&self, index: usize) -> u128 {
        self.regs[index].into()
    }

    fn get_be(&self, index: usize) -> [u16; 8] {
        self.regs[index].to_be_array()
    }

    fn get_le(&self, index: usize) -> [u16; 8] {
        self.regs[index].to_le_array()
    }

    fn broadcast_le(&self, index: usize, element: usize) -> [u16; 8] {
        self.regs[index].broadcast(element).to_le_array()
    }

    fn lane(&self, index: usize, element: usize) -> u16 {
        self.regs[index].lane(element)
    }

    fn set_u8(&mut self, index: usize, element: usize, value: u8) {
        let reg = &mut self.regs[index];
        reg.write(element, value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_u16(&mut self, index: usize, element: usize, value: u16) {
        let reg = &mut self.regs[index];
        reg.write(element, value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_u32(&mut self, index: usize, element: usize, value: u32) {
        let reg = &mut self.regs[index];
        reg.write(element, value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_u64(&mut self, index: usize, element: usize, value: u64) {
        let reg = &mut self.regs[index];
        reg.write(element, value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_u128(&mut self, index: usize, value: u128) {
        let reg = &mut self.regs[index];
        *reg = Vector::from(value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_be(&mut self, index: usize, value: [u16; 8]) {
        let reg = &mut self.regs[index];
        *reg = Vector::from_be_array(value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_le(&mut self, index: usize, value: [u16; 8]) {
        let reg = &mut self.regs[index];
        *reg = Vector::from_le_array(value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn set_lane(&mut self, index: usize, element: usize, value: u16) {
        let reg = &mut self.regs[index];
        reg.set_lane(element, value);
        trace!("  V{:02}: {}", index, reg);
    }

    fn acc_le(&self) -> [u64; 8] {
        self.accumulator
    }

    fn set_acc_le(&mut self, value: [u64; 8]) {
        let acc = &mut self.accumulator;

        *acc = value.map(|element| (((element as i64) << 16) >> 16) as u64);

        trace!(
            "  ACC: {:012X} {:012X} {:012X} {:012X} {:012X} {:012X} {:012X} {:012X}",
            acc[7],
            acc[6],
            acc[5],
            acc[4],
            acc[3],
            acc[2],
            acc[1],
            acc[0]
        );
    }
}

impl mips::Cp2 for Cp2 {
    fn mfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let op = Move::from(word);

        trace!(
            "{:08X} MFC2 {}, V{:02}[{}]",
            core.pc(),
            GPR[op.rt()],
            op.vs(),
            op.element(),
        );

        let index = op.element();
        let value = core.cp2().get_u16(op.vs(), index);
        core.setw(op.rt(), value as i16 as u32);
    }

    fn mtc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let op = Move::from(word);

        trace!(
            "{:08X} MTC2 {}, V{:02}[{}]",
            core.pc(),
            GPR[op.rt()],
            op.vs(),
            op.element(),
        );

        let index = op.element();
        let value = core.getw(op.rt());
        core.cp2_mut().set_u16(op.vs(), index, value as u16);
    }

    fn cfc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let op = Move::from(word);

        trace!(
            "{:08X} CFC2 {}, {}",
            core.pc(),
            GPR[op.rt()],
            CTRL_REGS[op.vs()],
        );

        let cp2 = core.cp2();

        let value = match op.vs() {
            0 => u16::from_le_bytes([cp2.carry.into_inner()[0], cp2.not_equal.into_inner()[0]]),
            1 => u16::from_le_bytes([
                cp2.compare.into_inner()[0],
                cp2.clip_compare.into_inner()[0],
            ]),
            2 => u16::from_le_bytes([cp2.compare_extension.into_inner()[0], 0]),
            reg => {
                warn!("RSP CP2 Control Register Read: {}", CTRL_REGS[reg]);
                0
            }
        };

        core.setw(op.rt(), value as i16 as u32);
    }

    fn ctc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        let op = Move::from(word);

        trace!(
            "{:08X} MTC2 {}, {}",
            core.pc(),
            GPR[op.rt()],
            CTRL_REGS[op.vs()],
        );

        let value = core.getw(op.rt());
        let cp2 = core.cp2_mut();

        match op.vs() {
            0 => {
                cp2.carry = [value as u8].into();
                cp2.not_equal = [(value >> 8) as u8].into();
            }
            1 => {
                cp2.compare = [value as u8].into();
                cp2.clip_compare = [(value >> 8) as u8].into();
            }
            2 => cp2.compare_extension = [value as u8].into(),
            reg => warn!("RSP CP2 Control Register Write: {}", CTRL_REGS[reg]),
        };

        let index = op.element();
        core.cp2_mut().set_u16(op.vs(), index, value as u16);
    }

    fn lwc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        match (word >> 11) & 0x1f {
            0x00 => load_store::lbv(core, word),
            0x01 => load_store::lsv(core, word),
            0x02 => load_store::llv(core, word),
            0x03 => load_store::ldv(core, word),
            0x04 => load_store::lqv(core, word),
            0x05 => load_store::lrv(core, word),
            0x06 => load_store::lpv(core, word),
            0x07 => load_store::luv(core, word),
            0x0b => load_store::ltv(core, word),
            opcode => unimplemented!("RSP LWC2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
        }
    }

    fn swc2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        match (word >> 11) & 0x1f {
            0x00 => load_store::sbv(core, word),
            0x01 => load_store::ssv(core, word),
            0x02 => load_store::slv(core, word),
            0x03 => load_store::sdv(core, word),
            0x04 => load_store::sqv(core, word),
            0x05 => load_store::srv(core, word),
            0x06 => load_store::spv(core, word),
            0x07 => load_store::suv(core, word),
            0x0b => load_store::stv(core, word),
            opcode => unimplemented!("RSP SWC2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
        }
    }

    fn cop2(core: &mut Core<impl Bus<Cp2 = Self>>, word: u32) {
        match word & 0x3f {
            0x00 => compute::vmulf(core, word),
            0x01 => compute::vmulu(core, word),
            0x04 => compute::vmudl(core, word),
            0x05 => compute::vmudm(core, word),
            0x06 => compute::vmudn(core, word),
            0x07 => compute::vmudh(core, word),
            0x08 => compute::vmacf(core, word),
            0x09 => compute::vmacu(core, word),
            0x0c => compute::vmadl(core, word),
            0x0d => compute::vmadm(core, word),
            0x0e => compute::vmadn(core, word),
            0x0f => compute::vmadh(core, word),
            0x10 => compute::vadd(core, word),
            0x11 => compute::vsub(core, word),
            0x13 => compute::vabs(core, word),
            0x14 => compute::vaddc(core, word),
            0x15 => compute::vsubc(core, word),
            0x1d => compute::vsar(core, word),
            0x20 => select::vlt(core, word),
            0x21 => select::veq(core, word),
            0x22 => select::vne(core, word),
            0x23 => select::vge(core, word),
            0x24 => select::vcl(core, word),
            0x25 => select::vch(core, word),
            0x26 => select::vcr(core, word),
            0x27 => select::vmrg(core, word),
            0x28 => compute::vand(core, word),
            0x29 => compute::vnand(core, word),
            0x2a => compute::vor(core, word),
            0x2b => compute::vnor(core, word),
            0x2c => compute::vxor(core, word),
            0x2d => compute::vnxor(core, word),
            0x30 => single_lane::vrcp(core, word),
            0x31 => single_lane::vrcpl(core, word),
            0x32 => single_lane::vrcph(core, word),
            0x33 => single_lane::vmov(core, word),
            0x34 => single_lane::vrsq(core, word),
            0x35 => single_lane::vrsql(core, word),
            0x36 => single_lane::vrsqh(core, word),
            0x37 => single_lane::vnop(core, word),
            0x3f => single_lane::vnull(core, word),
            opcode => unimplemented!("RSP COP2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
        }
    }
}

#[bitfield(u32)]
struct Move {
    #[bits(7)]
    __: u32,
    #[bits(4)]
    element: usize,
    #[bits(5)]
    vs: usize,
    #[bits(5)]
    rt: usize,
    #[bits(5)]
    opcode: u32,
    #[bits(6)]
    __: u32,
}

fn reciprocal(index: usize) -> u16 {
    if index == 0 {
        return 0xffff;
    }

    ((((1u64 << 34) / (index as u64 + 512)) + 1) >> 8) as u16
}

fn inv_sqrt(index: usize) -> u16 {
    let input = (index as u64 + 512) >> (index as u64 & 1);
    let mut result = 1u64 << 17;

    while (input * (result + 1) * (result + 1)) < (1u64 << 44) {
        result += 1;
    }

    (result >> 1) as u16
}

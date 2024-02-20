use crate::util::memory::Value;
use bitflags::bitflags;
use size::Size;
use std::mem;
use tracing::trace;

mod condition;
mod instruction;
mod operator;
mod size;

bitflags! {
    pub struct Interrupt: u32 {
        const RESET = 0x0100;
    }
}

pub trait Bus {
    fn read<T: Value>(&self, address: u32) -> T;
    fn write<T: Value>(&mut self, address: u32, value: T);
}

pub struct Flags {
    x: bool,
    n: u8,
    z: u32,
    v: u8,
    c: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Supervisor,
    User,
}

pub struct Core<T: Bus> {
    dreg: [u32; 8],
    areg: [u32; 8],
    pc: u32,
    sp_shadow: u32,
    flags: Flags,
    mode: Mode,
    int_level: u8,
    interrupt: Interrupt,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            dreg: [0; 8],
            areg: [0; 8],
            pc: 0,
            sp_shadow: 0,
            flags: Flags {
                x: false,
                n: 0,
                z: u32::MAX,
                v: 0,
                c: false,
            },
            mode: Mode::Supervisor,
            int_level: 0,
            interrupt: Interrupt::RESET,
            bus,
        }
    }

    pub fn step(&mut self) {
        use condition as cond;
        use instruction as instr;
        use operator as op;

        if !self.interrupt.is_empty() {
            if self.interrupt.contains(Interrupt::RESET) {
                instr::reset(self);
            } else {
                unimplemented!("Interrupt types other than reset");
            }

            self.interrupt = Interrupt::empty();
            return;
        }

        let word: u16 = self.next();

        #[allow(clippy::unusual_byte_groupings)]
        match word >> 6 {
            // Immediate Value Operations
            // 0b0000_0000_00 => instr::immediate::<op::Or, u8>(self, word),
            // 0b0000_0000_01 => instr::immediate::<op::Or, u16>(self, word),
            // 0b0000_0000_10 => instr::immediate::<op::Or, u32>(self, word),
            0b0000_0010_00 => instr::immediate::<op::And, u8>(self, word),
            0b0000_0010_01 => instr::immediate::<op::And, u16>(self, word),
            0b0000_0010_10 => instr::immediate::<op::And, u32>(self, word),
            // 0b0000_0100_00 => instr::immediate::<op::Add, u8>(self, word),
            // 0b0000_0100_01 => instr::immediate::<op::Add, u16>(self, word),
            // 0b0000_0100_10 => instr::immediate::<op::Add, u32>(self, word),
            // 0b0000_0110_00 => instr::immediate::<op::Sub, u8>(self, word),
            // 0b0000_0110_01 => instr::immediate::<op::Sub, u16>(self, word),
            // 0b0000_0110_10 => instr::immediate::<op::Sub, u32>(self, word),
            // 0b0000_1010_00 => instr::immediate::<op::Eor, u8>(self, word),
            // 0b0000_1010_01 => instr::immediate::<op::Eor, u16>(self, word),
            // 0b0000_1010_10 => instr::immediate::<op::Eor, u32>(self, word),

            // MOVEA
            0b0001_0000_01 | 0b0001_0010_01 | 0b0001_0100_01 | 0b0001_0110_01 | 0b0001_1000_01
            | 0b0001_1010_01 | 0b0001_1100_01 | 0b0001_1110_01 => instr::movea::<u8>(self, word),
            0b0010_0000_01 | 0b0010_0010_01 | 0b0010_0100_01 | 0b0010_0110_01 | 0b0010_1000_01
            | 0b0010_1010_01 | 0b0010_1100_01 | 0b0010_1110_01 => instr::movea::<u16>(self, word),
            0b0011_0000_01 | 0b0011_0010_01 | 0b0011_0100_01 | 0b0011_0110_01 | 0b0011_1000_01
            | 0b0011_1010_01 | 0b0011_1100_01 | 0b0011_1110_01 => instr::movea::<u32>(self, word),

            // MOVE
            0b0001_0000_00..=0b0001_1111_11 => instr::move_::<u8>(self, word),
            0b0010_0000_00..=0b0010_1111_11 => instr::move_::<u16>(self, word),
            0b0011_0000_00..=0b0011_1111_11 => instr::move_::<u32>(self, word),

            // 0b0100 (Unary/Misc)
            0b0100_1010_00 => instr::tst::<u8>(self, word),
            0b0100_1010_01 => instr::tst::<u16>(self, word),
            0b0100_1010_10 => instr::tst::<u32>(self, word),

            // Branches
            0b0110_0000_00..=0b0110_0000_11 => instr::bra(self, word),
            //0b0110_0001_00..=0b0110_0001_11 => instr::bsr(self, word),
            0b0110_0010_00..=0b0110_0010_11 => instr::bcc::<cond::HI>(self, word),
            0b0110_0011_00..=0b0110_0011_11 => instr::bcc::<cond::LS>(self, word),
            0b0110_0100_00..=0b0110_0100_11 => instr::bcc::<cond::CC>(self, word),
            0b0110_0101_00..=0b0110_0101_11 => instr::bcc::<cond::CS>(self, word),
            0b0110_0110_00..=0b0110_0110_11 => instr::bcc::<cond::NE>(self, word),
            0b0110_0111_00..=0b0110_0111_11 => instr::bcc::<cond::EQ>(self, word),
            0b0110_1000_00..=0b0110_1000_11 => instr::bcc::<cond::VC>(self, word),
            0b0110_1001_00..=0b0110_1001_11 => instr::bcc::<cond::VS>(self, word),
            0b0110_1010_00..=0b0110_1010_11 => instr::bcc::<cond::PL>(self, word),
            0b0110_1011_00..=0b0110_1011_11 => instr::bcc::<cond::MI>(self, word),
            0b0110_1100_00..=0b0110_1100_11 => instr::bcc::<cond::GE>(self, word),
            0b0110_1101_00..=0b0110_1101_11 => instr::bcc::<cond::LT>(self, word),
            0b0110_1110_00..=0b0110_1110_11 => instr::bcc::<cond::GT>(self, word),
            0b0110_1111_00..=0b0110_1111_11 => instr::bcc::<cond::LE>(self, word),

            // Special encodings
            0b0100_0001_11 | 0b0100_0010_11 | 0b0100_0101_11 | 0b0100_0111_11 | 0b0100_1001_11
            | 0b0100_1011_11 | 0b0100_1101_11 | 0b0100_1111_11 => instr::lea(self, word),

            //0b0100_1000_10 => instr::movem_write::<u16>(self, word),
            //0b0100_1000_11 => instr::movem_write::<u32>(self, word),
            0b0100_1100_10 => instr::movem_read::<u16>(self, word),
            0b0100_1100_11 => instr::movem_read::<u32>(self, word),

            0b0111_0000_00..=0b0111_0000_11
            | 0b0111_0010_00..=0b0111_0010_11
            | 0b0111_0100_00..=0b0111_0100_11
            | 0b0111_0110_00..=0b0111_0110_11
            | 0b0111_1000_00..=0b0111_1000_11
            | 0b0111_1010_00..=0b0111_1010_11
            | 0b0111_1100_00..=0b0111_1100_11
            | 0b0111_1110_00..=0b0111_1110_11 => instr::moveq(self, word),

            _ => unimplemented!(
                "M68000 Opcode: {:04b}_{:04b}_{:02b}",
                (word >> 12) & 15,
                (word >> 8) & 15,
                (word >> 6) & 3
            ),
        }
    }

    fn dreg<U: Size>(&self, index: usize) -> U {
        U::dreg(self, index)
    }

    fn set_dreg<U: Size>(&mut self, index: usize, value: U) {
        U::set_dreg(self, index, value);
    }

    fn areg<U: Size>(&self, index: usize) -> U {
        U::areg(self, index)
    }

    fn set_areg<U: Size>(&mut self, index: usize, value: U) {
        U::set_areg(self, index, value);
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
        trace!("  PC: {:08X}", self.pc);
    }

    fn set_ccr(&mut self, cb: impl Fn(&mut Flags)) {
        cb(&mut self.flags);
        trace!(
            "  CCR: {}{}{}{}{}",
            if self.flags.x { 'X' } else { '-' },
            if (self.flags.n & 0x80) != 0 { 'N' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if (self.flags.v & 0x80) != 0 { 'V' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
        );
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        trace!("  Mode: {:?}", self.mode);
    }

    fn set_int_level(&mut self, int_level: u8) {
        self.int_level = int_level;
        trace!("  Interrupt Level: {}", self.int_level);
    }

    fn read<U: Size>(&self, address: u32) -> U {
        U::read(self, address)
    }

    fn write<U: Size>(&mut self, address: u32, value: U) {
        U::write(self, address, value);
    }

    fn modify<U: Size>(&mut self, address: u32, cb: impl Fn(&mut Core<T>, U) -> U) {
        let value = self.read(address);
        let result = cb(self, value);
        self.write(address, result);
    }

    fn next<U: Size>(&mut self) -> U {
        U::next(self)
    }
}

impl Flags {
    fn set_nz<T: Size>(&mut self, value: T) {
        self.n = (value >> (mem::size_of::<T>() * 8 - 8)).as_();
        self.z = value.as_();
    }
}

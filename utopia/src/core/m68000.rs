use crate::util::memory::Value;
use bitflags::bitflags;
use size::Size;
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
    n: bool,
    z: bool,
    v: bool,
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
                n: false,
                z: false,
                v: false,
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

            // Bit operations
            0b0000_1000_00 => instr::btst_static(self, word),
            // 0b0000_1000_01 => instr::bchg_static(self, word),
            // 0b0000_1000_10 => instr::bclr_static(self, word),
            // 0b0000_1000_11 => instr::bset_static(self, word),
            0b0000_0001_00 | 0b0000_0011_00 | 0b0000_0101_00 | 0b0000_0111_00 | 0b0000_1001_00
            | 0b0000_1011_00 | 0b0000_1101_00 | 0b0000_1111_00 => instr::btst_dynamic(self, word),
            // 0b0000_0001_01 | 0b0000_0011_01 | 0b0000_0101_01 | 0b0000_0111_01 | 0b0000_1001_01
            // | 0b0000_1011_01 | 0b0000_1101_01 | 0b0000_1111_01 => instr::bchg_dynamic(self, word),
            // 0b0000_0001_10 | 0b0000_0011_10 | 0b0000_0101_10 | 0b0000_0111_10 | 0b0000_1001_10
            // | 0b0000_1011_10 | 0b0000_1101_10 | 0b0000_1111_10 => instr::bclr_dynamic(self, word),
            // 0b0000_0001_11 | 0b0000_0011_11 | 0b0000_0101_11 | 0b0000_0111_11 | 0b0000_1001_11
            // | 0b0000_1011_11 | 0b0000_1101_11 | 0b0000_1111_11 => instr::bset_dynamic(self, word),

            // MOVEA
            0b0001_0000_01 | 0b0001_0010_01 | 0b0001_0100_01 | 0b0001_0110_01 | 0b0001_1000_01
            | 0b0001_1010_01 | 0b0001_1100_01 | 0b0001_1110_01 => instr::movea::<u8>(self, word),
            0b0010_0000_01 | 0b0010_0010_01 | 0b0010_0100_01 | 0b0010_0110_01 | 0b0010_1000_01
            | 0b0010_1010_01 | 0b0010_1100_01 | 0b0010_1110_01 => instr::movea::<u32>(self, word),
            0b0011_0000_01 | 0b0011_0010_01 | 0b0011_0100_01 | 0b0011_0110_01 | 0b0011_1000_01
            | 0b0011_1010_01 | 0b0011_1100_01 | 0b0011_1110_01 => instr::movea::<u16>(self, word),

            // MOVE
            0b0001_0000_00..=0b0001_1111_11 => instr::move_::<u8>(self, word),
            0b0010_0000_00..=0b0010_1111_11 => instr::move_::<u32>(self, word),
            0b0011_0000_00..=0b0011_1111_11 => instr::move_::<u16>(self, word),

            // 0b0100 (Unary/Misc)
            0b0100_0110_11 => instr::move_to_sr(self, word),

            0b0100_1010_00 => instr::tst::<u8>(self, word),
            0b0100_1010_01 => instr::tst::<u16>(self, word),
            0b0100_1010_10 => instr::tst::<u32>(self, word),

            0b0100_1110_01 => self.dispatch_special(word),

            0b0101_0000_11 => instr::scc_dbcc::<cond::T>(self, word),
            0b0101_0001_11 => instr::scc_dbcc::<cond::F>(self, word),
            0b0101_0010_11 => instr::scc_dbcc::<cond::HI>(self, word),
            0b0101_0011_11 => instr::scc_dbcc::<cond::LS>(self, word),
            0b0101_0100_11 => instr::scc_dbcc::<cond::CC>(self, word),
            0b0101_0101_11 => instr::scc_dbcc::<cond::CS>(self, word),
            0b0101_0110_11 => instr::scc_dbcc::<cond::NE>(self, word),
            0b0101_0111_11 => instr::scc_dbcc::<cond::EQ>(self, word),
            0b0101_1000_11 => instr::scc_dbcc::<cond::VC>(self, word),
            0b0101_1001_11 => instr::scc_dbcc::<cond::VS>(self, word),
            0b0101_1010_11 => instr::scc_dbcc::<cond::PL>(self, word),
            0b0101_1011_11 => instr::scc_dbcc::<cond::MI>(self, word),
            0b0101_1100_11 => instr::scc_dbcc::<cond::GE>(self, word),
            0b0101_1101_11 => instr::scc_dbcc::<cond::LT>(self, word),
            0b0101_1110_11 => instr::scc_dbcc::<cond::GT>(self, word),
            0b0101_1111_11 => instr::scc_dbcc::<cond::LE>(self, word),

            // Branches
            0b0110_0000_00..=0b0110_0000_11 => instr::bra(self, word),
            0b0110_0001_00..=0b0110_0001_11 => instr::bsr(self, word),
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
            0b0100_0001_11 | 0b0100_0011_11 | 0b0100_0101_11 | 0b0100_0111_11 | 0b0100_1001_11
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

            // CMP/CMPA/CMPM
            0b1011_0000_00 | 0b1011_0010_00 | 0b1011_0100_00 | 0b1011_0110_00 | 0b1011_1000_00
            | 0b1011_1010_00 | 0b1011_1100_00 | 0b1011_1110_00 => instr::cmp::<u8>(self, word),
            0b1011_0000_01 | 0b1011_0010_01 | 0b1011_0100_01 | 0b1011_0110_01 | 0b1011_1000_01
            | 0b1011_1010_01 | 0b1011_1100_01 | 0b1011_1110_01 => instr::cmp::<u16>(self, word),
            0b1011_0000_10 | 0b1011_0010_10 | 0b1011_0100_10 | 0b1011_0110_10 | 0b1011_1000_10
            | 0b1011_1010_10 | 0b1011_1100_10 | 0b1011_1110_10 => instr::cmp::<u32>(self, word),

            // ADD/ADDX/ADDA
            0b1101_0000_00 | 0b1101_0010_00 | 0b1101_0100_00 | 0b1101_0110_00 | 0b1101_1000_00
            | 0b1101_1010_00 | 0b1101_1100_00 | 0b1101_1110_00 => {
                instr::read::<op::Add, u8>(self, word)
            }
            0b1101_0000_01 | 0b1101_0010_01 | 0b1101_0100_01 | 0b1101_0110_01 | 0b1101_1000_01
            | 0b1101_1010_01 | 0b1101_1100_01 | 0b1101_1110_01 => {
                instr::read::<op::Add, u16>(self, word)
            }
            0b1101_0000_10 | 0b1101_0010_10 | 0b1101_0100_10 | 0b1101_0110_10 | 0b1101_1000_10
            | 0b1101_1010_10 | 0b1101_1100_10 | 0b1101_1110_10 => {
                instr::read::<op::Add, u32>(self, word)
            }

            // 0b1101_0001_00 | 0b1101_0011_00 | 0b1101_0101_00 | 0b1101_0111_00 | 0b1101_1001_00
            // | 0b1101_1011_00 | 0b1101_1101_00 | 0b1101_1111_00 => {
            //     instr::write::<op::Add, u8>(self, word)
            // }
            // 0b1101_0001_01 | 0b1101_0011_01 | 0b1101_0101_01 | 0b1101_0111_01 | 0b1101_1001_01
            // | 0b1101_1011_01 | 0b1101_1101_01 | 0b1101_1111_01 => {
            //     instr::write::<op::Add, u16>(self, word)
            // }
            // 0b1101_0001_10 | 0b1101_0011_10 | 0b1101_0101_10 | 0b1101_0111_10 | 0b1101_1001_10
            // | 0b1101_1011_10 | 0b1101_1101_10 | 0b1101_1111_10 => {
            //     instr::write::<op::Add, u32>(self, word)
            // }
            _ => unimplemented!(
                "M68000 Opcode: {:04b}_{:04b}_{:02b}",
                (word >> 12) & 15,
                (word >> 8) & 15,
                (word >> 6) & 3
            ),
        }
    }

    fn dispatch_special(&mut self, word: u16) {
        use instruction as instr;

        #[allow(clippy::unusual_byte_groupings)]
        match word & 0b111_111 {
            //0b100_000..=0b100_111 => instr::move_from_usp(self, word),
            0b100_000..=0b100_111 => instr::move_to_usp(self, word),
            _ => unimplemented!(
                "M68000 Special Opcode: {:03b}_{:03b}",
                (word >> 3) & 7,
                word & 7
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

    fn set_usp(&mut self, value: u32) {
        if self.mode != Mode::Supervisor {
            panic!("Attempted to set USP outside of Supervisor mode");
        }

        self.sp_shadow = value;
        trace!("  USP: {:08X}", self.sp_shadow);
    }

    fn set_sr(&mut self, value: u16) {
        if self.mode != Mode::Supervisor {
            panic!("Attempted to set SR outside of Supervisor mode");
        }

        // TODO: Trace mode

        self.set_mode(if (value & 0x2000) != 0 {
            Mode::Supervisor
        } else {
            Mode::User
        });

        self.set_int_level(((value & 0x0700) >> 8) as u8);

        self.set_ccr(|flags| {
            flags.x = (value & 0x0010) != 0;
            flags.n = (value & 0x0008) != 0;
            flags.z = (value & 0x0004) != 0;
            flags.v = (value & 0x0002) != 0;
            flags.c = (value & 0x0001) != 0;
        });
    }

    fn set_ccr(&mut self, cb: impl Fn(&mut Flags)) {
        cb(&mut self.flags);
        trace!(
            "  CCR: {}{}{}{}{}",
            if self.flags.x { 'X' } else { '-' },
            if self.flags.n { 'N' } else { '-' },
            if self.flags.z { 'Z' } else { '-' },
            if self.flags.v { 'V' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
        );
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;

        if self.mode == Mode::User {
            todo!("Switch to user mode");
        }

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
        self.n = (value & T::SIGN_BIT) != T::zero();
        self.z = value == T::zero();
    }
}

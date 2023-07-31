use crate::util::facade::Value;
use tracing::debug;

mod instruction;
mod operator;

#[rustfmt::skip]
const REGS: [&'static str; 16] = [
    "R0", "R1", "R2", "R3",
    "R4", "R5", "R6", "R7",
    "R8", "R9", "R10", "R11",
    "R12", "SP", "LR", "PC",
];

pub trait Bus {
    fn read<T: Value>(&mut self, address: u32) -> T;
    fn write<T: Value>(&mut self, address: u32, value: T);
}

#[repr(u8)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
enum Mode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    #[default]
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

#[derive(Clone, Default)]
struct Cpsr {
    n: bool,
    z: bool,
    c: bool,
    v: bool,
    i: bool,
    f: bool,
    t: bool,
    m: Mode,
    reserved: u32,
}

#[derive(Clone, Default)]
struct Spsr {
    fiq: u32,
    svc: u32,
    abt: u32,
    irq: u32,
    und: u32,
}

#[derive(Clone, Default)]
struct Bank {
    usr: [u32; 7],
    fiq: [u32; 7],
    svc: [u32; 2],
    abt: [u32; 2],
    irq: [u32; 2],
    und: [u32; 2],
}

pub struct Core<T: Bus> {
    bus: T,
    pc: u32,
    regs: [u32; 16],
    cpsr: Cpsr,
    spsr: Spsr,
    bank: Bank,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            pc: 0,
            regs: [0; 16],
            cpsr: Cpsr {
                f: true,
                i: true,
                ..Default::default()
            },
            spsr: Default::default(),
            bank: Default::default(),
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;
        use operator as op;

        assert!((self.pc & 3) == 0);

        let pc = self.pc;
        let word = self.bus.read::<u32>(self.pc);
        self.pc = self.pc.wrapping_add(4);

        let (name, result) = self.apply_condition(word);

        if !result {
            debug!("{:08X}: ({}: Skipped)", self.pc, name);
            return;
        }

        match (word >> 20) & 0xff {
            0x12 => instr::msr_register::<false>(self, pc, word),
            0x16 => instr::msr_register::<true>(self, pc, word),

            //0x20 => instr::binary_immediate::<op::And, false>(self, pc, word),
            //0x21 => instr::binary_immediate::<op::And, true>(self, pc, word),
            //0x22 => instr::binary_immediate::<op::Eor, false>(self, pc, word),
            //0x23 => instr::binary_immediate::<op::Eor, true>(self, pc, word),
            //0x24 => instr::binary_immediate::<op::Sub, false>(self, pc, word),
            //0x25 => instr::binary_immediate::<op::Sub, true>(self, pc, word),
            //0x26 => instr::binary_immediate::<op::Rsb, false>(self, pc, word),
            //0x27 => instr::binary_immediate::<op::Rsb, true>(self, pc, word),
            0x28 => instr::binary_immediate::<op::Add, false>(self, pc, word),
            0x29 => instr::binary_immediate::<op::Add, true>(self, pc, word),
            0x2a => instr::binary_immediate::<op::Adc, false>(self, pc, word),
            0x2b => instr::binary_immediate::<op::Adc, true>(self, pc, word),
            //0x2c => instr::binary_immediate::<op::Sbc, false>(self, pc, word),
            //0x2d => instr::binary_immediate::<op::Sbc, true>(self, pc, word),
            //0x2e => instr::binary_immediate::<op::Rsc, false>(self, pc, word),
            //0x2f => instr::binary_immediate::<op::Rsc, true>(self, pc, word),
            0x31 => instr::compare_immediate::<op::Tst>(self, pc, word),
            0x33 => instr::compare_immediate::<op::Teq>(self, pc, word),
            0x35 => instr::compare_immediate::<op::Cmp>(self, pc, word),
            0x37 => instr::compare_immediate::<op::Cmn>(self, pc, word),

            //0x38 => instr::binary_immediate::<op::Or, false>(self, pc, word),
            //0x39 => instr::binary_immediate::<op::Or, true>(self, pc, word),
            0x3a => instr::move_immediate::<op::Mov, false>(self, pc, word),
            0x3b => instr::move_immediate::<op::Mov, true>(self, pc, word),
            //0x3c => instr::binary_immediate::<op::Bic, false>(self, pc, word),
            //0x3d => instr::binary_immediate::<op::Bic, true>(self, pc, word),
            //0x3e => instr::move_immediate::<op::Mvn, false>(self, pc, word),
            //0x3f => instr::move_immediate::<op::Mvn, true>(self, pc, word),
            0x40 => instr::str_immediate::<false, 0b000>(self, pc, word),
            0x41 => instr::ldr_immediate::<false, 0b000>(self, pc, word),
            0x42 => instr::str_immediate::<false, 0b001>(self, pc, word),
            0x43 => instr::ldr_immediate::<false, 0b001>(self, pc, word),
            0x44 => instr::str_immediate::<true, 0b000>(self, pc, word),
            0x45 => instr::ldr_immediate::<true, 0b000>(self, pc, word),
            0x46 => instr::str_immediate::<true, 0b001>(self, pc, word),
            0x47 => instr::ldr_immediate::<true, 0b001>(self, pc, word),

            0x48 => instr::str_immediate::<false, 0b010>(self, pc, word),
            0x49 => instr::ldr_immediate::<false, 0b010>(self, pc, word),
            0x4a => instr::str_immediate::<false, 0b011>(self, pc, word),
            0x4b => instr::ldr_immediate::<false, 0b011>(self, pc, word),
            0x4c => instr::str_immediate::<true, 0b010>(self, pc, word),
            0x4d => instr::ldr_immediate::<true, 0b010>(self, pc, word),
            0x4e => instr::str_immediate::<true, 0b011>(self, pc, word),
            0x4f => instr::ldr_immediate::<true, 0b011>(self, pc, word),

            0x50 => instr::str_immediate::<false, 0b100>(self, pc, word),
            0x51 => instr::ldr_immediate::<false, 0b100>(self, pc, word),
            0x52 => instr::str_immediate::<false, 0b101>(self, pc, word),
            0x53 => instr::ldr_immediate::<false, 0b101>(self, pc, word),
            0x54 => instr::str_immediate::<true, 0b100>(self, pc, word),
            0x55 => instr::ldr_immediate::<true, 0b100>(self, pc, word),
            0x56 => instr::str_immediate::<true, 0b101>(self, pc, word),
            0x57 => instr::ldr_immediate::<true, 0b101>(self, pc, word),

            0x58 => instr::str_immediate::<false, 0b110>(self, pc, word),
            0x59 => instr::ldr_immediate::<false, 0b110>(self, pc, word),
            0x5a => instr::str_immediate::<false, 0b111>(self, pc, word),
            0x5b => instr::ldr_immediate::<false, 0b111>(self, pc, word),
            0x5c => instr::str_immediate::<true, 0b110>(self, pc, word),
            0x5d => instr::ldr_immediate::<true, 0b110>(self, pc, word),
            0x5e => instr::str_immediate::<true, 0b111>(self, pc, word),
            0x5f => instr::ldr_immediate::<true, 0b111>(self, pc, word),

            0xa0..=0xaf => instr::branch::<false>(self, pc, word),
            0xb0..=0xbf => instr::branch::<true>(self, pc, word),

            opcode => todo!(
                "ARM7 Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
                opcode,
                self.pc
            ),
        }
    }

    fn apply_condition(&self, word: u32) -> (&'static str, bool) {
        match word >> 28 {
            0b0000 => ("EQ", self.cpsr.z),
            0b0001 => ("NE", !self.cpsr.z),
            0b0010 => ("CS", self.cpsr.c),
            0b0011 => ("CC", !self.cpsr.c),
            0b0100 => ("MI", self.cpsr.n),
            0b0101 => ("PL", !self.cpsr.n),
            0b0110 => ("VS", self.cpsr.v),
            0b0111 => ("VC", !self.cpsr.v),
            0b1000 => ("HI", !self.cpsr.z && self.cpsr.c),
            0b1001 => ("LS", self.cpsr.z || !self.cpsr.c),
            0b1010 => ("GE", self.cpsr.n == self.cpsr.v),
            0b1011 => ("LT", self.cpsr.n != self.cpsr.v),
            0b1100 => ("GT", !self.cpsr.z && self.cpsr.n == self.cpsr.v),
            0b1101 => ("LE", self.cpsr.z || self.cpsr.n != self.cpsr.z),
            0b1110 => ("", true),
            code => unimplemented!("Condition code {:04b}", code),
        }
    }

    fn read_byte(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:02X}", address, value);
        value
    }

    fn read_word(&mut self, address: u32) -> u32 {
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:08X}", address, value);
        value
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        debug!("  [{:08X}] <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        debug!("  [{:08X}] <= {:08X}", address, value);
        self.bus.write(address, value);
    }

    fn get(&self, reg: usize) -> u32 {
        if reg == 15 {
            self.pc.wrapping_add(4)
        } else {
            self.regs[reg]
        }
    }

    fn set(&mut self, reg: usize, value: u32) {
        if reg == 15 {
            todo!("PC set");
        }

        self.regs[reg] = value;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn cpsr_to_u32(&self) -> u32 {
        let mut value = self.cpsr.reserved | self.cpsr.m as u32;
        value |= if self.cpsr.n { 0x8000_0000 } else { 0 };
        value |= if self.cpsr.z { 0x4000_0000 } else { 0 };
        value |= if self.cpsr.c { 0x2000_0000 } else { 0 };
        value |= if self.cpsr.v { 0x1000_0000 } else { 0 };
        value |= if self.cpsr.i { 0x80 } else { 0 };
        value |= if self.cpsr.f { 0x40 } else { 0 };
        value |= if self.cpsr.t { 0x20 } else { 0 };
        value
    }

    fn cpsr_from_u32(&mut self, value: u32, control: bool) {
        self.cpsr.n = (value & 0x8000_0000) != 0;
        self.cpsr.z = (value & 0x4000_0000) != 0;
        self.cpsr.c = (value & 0x2000_0000) != 0;
        self.cpsr.v = (value & 0x1000_0000) != 0;

        if control && self.cpsr.m != Mode::User {
            self.cpsr.reserved = value & 0x0fff_ff00;
            self.cpsr.i = (value & 0x80) != 0;
            self.cpsr.f = (value & 0x40) != 0;
            self.cpsr.t = (value & 0x20) != 0;

            self.set_mode(match value & 0x1f {
                0b10000 => Mode::User,
                0b10001 => Mode::Fiq,
                0b10010 => Mode::Irq,
                0b10011 => Mode::Supervisor,
                0b10111 => Mode::Abort,
                0b11011 => Mode::Undefined,
                0b11111 => Mode::System,
                mode => panic!("Invalid mode: {:05b}", mode),
            });
        }

        debug!("  CPSR: {:08X}", self.cpsr_to_u32());
    }

    fn spsr_to_u32(&self) -> u32 {
        match self.cpsr.m {
            Mode::Fiq => self.spsr.fiq,
            Mode::Irq => self.spsr.irq,
            Mode::Supervisor => self.spsr.svc,
            Mode::Abort => self.spsr.abt,
            Mode::Undefined => self.spsr.und,
            mode => panic!("No SPSR defined for mode: {:?}", mode),
        }
    }

    fn spsr_from_u32(&mut self, value: u32, control: bool) {
        let spsr = match self.cpsr.m {
            Mode::Fiq => &mut self.spsr.fiq,
            Mode::Irq => &mut self.spsr.irq,
            Mode::Supervisor => &mut self.spsr.svc,
            Mode::Abort => &mut self.spsr.abt,
            Mode::Undefined => &mut self.spsr.und,
            mode => panic!("No SPSR defined for mode: {:?}", mode),
        };

        if control {
            *spsr = value;
        } else {
            *spsr = (value & 0xf000_0000) | (*spsr & 0x0fff_ffff);
        }

        debug!("  SPSR: {:08X}", spsr);
    }

    fn set_nz(&mut self, value: u32) {
        self.cpsr.n = (value & 0x8000_0000) != 0;
        self.cpsr.z = value == 0;
    }

    fn add_with_carry<const SET_FLAGS: bool>(&mut self, lhs: u32, rhs: u32, carry: bool) -> u32 {
        let result = lhs.wrapping_add(rhs).wrapping_add(carry as u32);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ result);

        if SET_FLAGS {
            self.set_nz(result);
            self.cpsr.c = ((carries ^ overflow) & 0x8000_0000) != 0;
            self.cpsr.v = (overflow & 0x8000_0000) != 0;
        }

        result
    }

    fn set_mode(&mut self, mode: Mode) {
        if mode == self.cpsr.m {
            return;
        }

        debug!("  Mode: {:?}", mode);
        debug!("  Stored: {:X?}", &self.regs[8..=14]);

        match self.cpsr.m {
            Mode::User | Mode::System => self.bank.usr.copy_from_slice(&self.regs[8..=14]),
            Mode::Fiq => self.bank.fiq.copy_from_slice(&self.regs[8..=14]),
            Mode::Irq => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.irq.copy_from_slice(&self.regs[13..=14]);
            }
            Mode::Supervisor => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.svc.copy_from_slice(&self.regs[13..=14]);
            }
            Mode::Abort => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.abt.copy_from_slice(&self.regs[13..=14]);
            }
            Mode::Undefined => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.und.copy_from_slice(&self.regs[13..=14]);
            }
        }

        self.cpsr.m = mode;

        match self.cpsr.m {
            Mode::User | Mode::System => self.regs[8..=14].copy_from_slice(&self.bank.usr),
            Mode::Fiq => self.regs[8..=14].copy_from_slice(&self.bank.fiq),
            Mode::Irq => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.irq);
            }
            Mode::Supervisor => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.svc);
            }
            Mode::Abort => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.abt);
            }
            Mode::Undefined => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.und);
            }
        }

        debug!("  Loaded: {:X?}", &self.regs[8..=14]);
    }
}

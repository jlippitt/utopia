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

#[derive(Clone, Default)]
struct Psr {
    n: bool,
    z: bool,
    c: bool,
    v: bool,
    i: bool,
    f: bool,
    t: bool,
    m: u8,
}

pub struct Core<T: Bus> {
    bus: T,
    pc: u32,
    regs: [u32; 16],
    cpsr: Psr,
    spsr: Psr,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            pc: 0,
            regs: [0; 16],
            cpsr: Default::default(),
            spsr: Default::default(),
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

            0x31 => instr::compare_immediate::<op::Tst>(self, pc, word),
            0x33 => instr::compare_immediate::<op::Teq>(self, pc, word),
            0x35 => instr::compare_immediate::<op::Cmp>(self, pc, word),
            0x37 => instr::compare_immediate::<op::Cmn>(self, pc, word),

            0x3a => instr::move_immediate::<op::Mov, false>(self, pc, word),

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
            todo!("PC get");
        }

        self.regs[reg]
    }

    fn set(&mut self, reg: usize, value: u32) {
        if reg == 15 {
            todo!("PC set");
        }

        self.regs[reg] = value;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn get_psr<const SUPER: bool>(&self) -> u32 {
        let psr = if SUPER { &self.spsr } else { &self.cpsr };
        let mut value = psr.m as u32;
        value |= if psr.n { 0x8000_0000 } else { 0 };
        value |= if psr.z { 0x4000_0000 } else { 0 };
        value |= if psr.c { 0x2000_0000 } else { 0 };
        value |= if psr.v { 0x1000_0000 } else { 0 };
        value |= if psr.i { 0x80 } else { 0 };
        value |= if psr.f { 0x40 } else { 0 };
        value |= if psr.t { 0x20 } else { 0 };
        value
    }

    fn set_psr<const SUPER: bool>(&mut self, value: u32, control: bool) {
        let psr = if SUPER {
            &mut self.spsr
        } else {
            &mut self.cpsr
        };

        psr.n = (value & 0x8000_0000) != 0;
        psr.z = (value & 0x4000_0000) != 0;
        psr.c = (value & 0x2000_0000) != 0;
        psr.v = (value & 0x1000_0000) != 0;

        if control {
            psr.i = (value & 0x80) != 0;
            psr.f = (value & 0x40) != 0;
            psr.t = (value & 0x20) != 0;
            psr.m = value as u8 & 0x1f;
        }

        debug!(
            "  {}PSR: {:08X}",
            if SUPER { 'S' } else { 'C' },
            self.get_psr::<SUPER>()
        )
    }

    fn set_nz(&mut self, value: u32) {
        self.cpsr.n = (value & 0x8000_0000) != 0;
        self.cpsr.z = value == 0;
    }

    fn add_with_carry(&mut self, lhs: u32, rhs: u32, carry: bool) -> u32 {
        let result = lhs.wrapping_add(rhs).wrapping_add(carry as u32);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ result);
        self.set_nz(result);
        self.cpsr.c = ((carries ^ overflow) & 0x8000_0000) != 0;
        self.cpsr.v = (overflow & 0x8000_0000) != 0;
        result
    }
}

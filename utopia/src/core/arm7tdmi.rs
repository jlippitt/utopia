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
}

struct Flags {
    n: bool,
    z: bool,
    c: bool,
    v: bool,
}

pub struct Core<T: Bus> {
    bus: T,
    pc: u32,
    regs: [u32; 16],
    flags: Flags,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            pc: 0,
            regs: [0; 16],
            flags: Flags {
                n: false,
                z: false,
                c: false,
                v: false,
            },
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
        }

        match (word >> 20) & 0xff {
            0x35 => instr::compare_immediate::<op::Cmp>(self, pc, word),

            0x3a => instr::move_immediate::<op::Mov, false>(self, pc, word),

            //0x40 => instr::str_immediate::<0b000>(self, pc, word),
            0x41 => instr::ldr_immediate::<0b000>(self, pc, word),
            //0x42 => instr::str_immediate::<0b001>(self, pc, word),
            0x43 => instr::ldr_immediate::<0b001>(self, pc, word),
            //0x44 => instr::strb_immediate::<0b000>(self, pc, word),
            0x45 => instr::ldrb_immediate::<0b000>(self, pc, word),
            //0x46 => instr::strb_immediate::<0b001>(self, pc, word),
            0x47 => instr::ldrb_immediate::<0b001>(self, pc, word),

            //0x48 => instr::str_immediate::<0b010>(self, pc, word),
            0x49 => instr::ldr_immediate::<0b010>(self, pc, word),
            //0x4a => instr::str_immediate::<0b011>(self, pc, word),
            0x4b => instr::ldr_immediate::<0b011>(self, pc, word),
            //0x4c => instr::strb_immediate::<0b010>(self, pc, word),
            0x4d => instr::ldrb_immediate::<0b010>(self, pc, word),
            //0x4e => instr::strb_immediate::<0b011>(self, pc, word),
            0x4f => instr::ldrb_immediate::<0b011>(self, pc, word),

            //0x50 => instr::str_immediate::<0b100>(self, pc, word),
            0x51 => instr::ldr_immediate::<0b100>(self, pc, word),
            //0x52 => instr::str_immediate::<0b101>(self, pc, word),
            0x53 => instr::ldr_immediate::<0b101>(self, pc, word),
            //0x54 => instr::strb_immediate::<0b100>(self, pc, word),
            0x55 => instr::ldrb_immediate::<0b100>(self, pc, word),
            //0x56 => instr::strb_immediate::<0b101>(self, pc, word),
            0x57 => instr::ldrb_immediate::<0b101>(self, pc, word),

            //0x58 => instr::str_immediate::<0b110>(self, pc, word),
            0x59 => instr::ldr_immediate::<0b110>(self, pc, word),
            //0x5a => instr::str_immediate::<0b111>(self, pc, word),
            0x5b => instr::ldr_immediate::<0b111>(self, pc, word),
            //0x5c => instr::strb_immediate::<0b110>(self, pc, word),
            0x5d => instr::ldrb_immediate::<0b110>(self, pc, word),
            //0x5e => instr::strb_immediate::<0b111>(self, pc, word),
            0x5f => instr::ldrb_immediate::<0b111>(self, pc, word),
            0xa0 => instr::branch::<false>(self, pc, word),

            opcode => todo!(
                "ARM7 Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
                opcode,
                self.pc
            ),
        }
    }

    fn apply_condition(&self, word: u32) -> (&'static str, bool) {
        match word >> 28 {
            0b0000 => ("EQ", self.flags.z),
            0b0001 => ("NE", !self.flags.z),
            0b0010 => ("CS", self.flags.c),
            0b0011 => ("CC", !self.flags.c),
            0b0100 => ("MI", self.flags.n),
            0b0101 => ("PL", !self.flags.n),
            0b0110 => ("VS", self.flags.v),
            0b0111 => ("VC", !self.flags.v),
            0b1000 => ("HI", !self.flags.z && self.flags.c),
            0b1001 => ("LS", self.flags.z || !self.flags.c),
            0b1010 => ("GE", self.flags.n == self.flags.v),
            0b1011 => ("LT", self.flags.n != self.flags.v),
            0b1100 => ("GT", !self.flags.z && self.flags.n == self.flags.v),
            0b1101 => ("LE", self.flags.z || self.flags.n != self.flags.z),
            0b1110 => ("", true),
            code => unimplemented!("Condition code {:04b}", code),
        }
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

    fn set_nz(&mut self, value: u32) {
        self.flags.n = (value & 0x8000_0000) != 0;
        self.flags.z = value == 0;
    }

    fn add_with_carry(&mut self, lhs: u32, rhs: u32, carry: bool) -> u32 {
        let result = lhs.wrapping_add(rhs).wrapping_add(carry as u32);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ result);
        self.set_nz(result);
        self.flags.c = ((carries ^ overflow) & 0x8000_0000) != 0;
        self.flags.v = (overflow & 0x8000_0000) != 0;
        result
    }
}

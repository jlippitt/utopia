use crate::util::facade::Value;
use tracing::debug;

mod instruction;
mod operator;

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
            0b1110 => ("", true),
            code => panic!("Condition not yet implemented: {:04b}", code),
        }
    }

    fn get(&self, reg: u32) -> u32 {
        if reg == 15 {
            todo!("PC fetch");
        }

        self.regs[reg as usize]
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

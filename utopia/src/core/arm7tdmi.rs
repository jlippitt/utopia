use crate::util::facade::Value;
use tracing::debug;

mod instruction;

pub trait Bus {
    fn read<T: Value>(&mut self, address: u32) -> T;
}

pub struct Core<T: Bus> {
    bus: T,
    pc: u32,
    regs: [u32; 16],
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            pc: 0,
            regs: [0; 16],
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;

        assert!((self.pc & 3) == 0);
        let word = self.bus.read::<u32>(self.pc);
        let (name, result) = self.apply_condition(word);

        if !result {
            debug!("{:08X}: ({}: Skipped)", self.pc, name);
        }

        match (word >> 20) & 0xff {
            0xa0 => instr::branch::<false>(self, word),
            opcode => todo!(
                "ARM7 Opcode {0:02X} [{0:08b}] (PC: {1:08X})",
                opcode,
                self.pc
            ),
        }

        //self.pc = self.pc.wrapping_add(4);
    }

    fn apply_condition(&self, word: u32) -> (&'static str, bool) {
        match word >> 28 {
            0b1110 => ("", true),
            code => panic!("Condition not yet implemented: {:04b}", code),
        }
    }
}
